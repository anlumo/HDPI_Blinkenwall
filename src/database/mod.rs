use crate::server::ShaderData;
use git2::{BranchType, Commit, Error, ObjectType, Oid, Repository, Signature, TreeBuilder};
use serde_json::json;

pub struct Database {
    repository: Repository,
}

const BRANCH_PREFIX: &str = "shader-";

impl Database {
    pub fn new(path: &str) -> Database {
        let repo = Repository::open(path).unwrap();

        Database { repository: repo }
    }

    pub fn list(&self) -> Result<Vec<String>, Error> {
        match self.repository.branches(Some(BranchType::Local)) {
            Err(error) => Err(error),
            Ok(branches) => Ok(branches
                .filter(|opt| match opt {
                    Ok((ref branch, _)) => match branch.name() {
                        Ok(optname) => match optname {
                            Some(name) => name.starts_with(BRANCH_PREFIX),
                            None => false,
                        },
                        Err(_) => false,
                    },
                    Err(_) => false,
                })
                .map(|r| {
                    String::from(
                        r.unwrap()
                            .0
                            .name()
                            .unwrap()
                            .unwrap()
                            .split_at(BRANCH_PREFIX.len())
                            .1,
                    )
                })
                .collect()),
        }
    }

    pub fn read(&self, name: &str) -> Result<ShaderData, Error> {
        let branchobj = self
            .repository
            .revparse_single(&vec![BRANCH_PREFIX, name].join(""))?;
        let branch = branchobj.as_commit().unwrap();
        let tree = self
            .repository
            .find_object(branch.id(), Some(ObjectType::Commit))?
            .as_commit()
            .unwrap()
            .tree()?;
        let entry = tree.get_name("shader.txt");
        match entry {
            None => Err(Error::from_str("internal error")),
            Some(entry) => {
                let obj = entry.to_object(&self.repository)?;
                let file = obj.as_blob().unwrap();
                match String::from_utf8(Vec::from(file.content())) {
                    Ok(source) => match tree.get_name("metadata.json") {
                        None => Err(Error::from_str("internal error")),
                        Some(meta) => {
                            match serde_json::from_str(
                                &String::from_utf8(Vec::from(
                                    meta.to_object(&self.repository)?
                                        .as_blob()
                                        .unwrap()
                                        .content(),
                                ))
                                .unwrap(),
                            ) {
                                Ok(obj) => {
                                    let obj: serde_json::Value = obj;
                                    if let (
                                        &serde_json::Value::String(ref title),
                                        &serde_json::Value::String(ref description),
                                    ) = (&obj["title"], &obj["description"])
                                    {
                                        Ok(ShaderData {
                                            title: title.clone(),
                                            description: description.clone(),
                                            source,
                                            commit: format!("{}", branch.id()),
                                        })
                                    } else {
                                        Err(Error::from_str("invalid format"))
                                    }
                                }
                                Err(error) => Err(Error::from_str(&format!("{}", error))),
                            }
                        }
                    },
                    Err(utf8err) => Err(Error::from_str(&format!("{}", utf8err.utf8_error()))),
                }
            }
        }
    }

    fn commit_treebuilder(
        &self,
        parent: Option<&Commit>,
        treebuilder: &TreeBuilder,
        message: &str,
    ) -> Result<Oid, Error> {
        let treeoid = treebuilder.write().unwrap();
        let treeobj = self
            .repository
            .find_object(treeoid, Some(ObjectType::Tree))
            .unwrap();
        let tree = treeobj.as_tree().unwrap();

        let signature = Signature::now("Blinkenwall", "blinkenwall@monitzer.com").unwrap();
        match parent {
            Some(parent) => {
                self.repository
                    .commit(None, &signature, &signature, message, tree, &[parent])
            }
            None => self
                .repository
                .commit(None, &signature, &signature, message, tree, &[]),
        }
    }

    fn create_commit(
        &self,
        branch: Option<&Commit>,
        data: &ShaderData,
        message: &str,
    ) -> Result<Oid, Error> {
        let source_bytes = data.source.as_bytes();
        let source_oid = self.repository.blob(source_bytes)?;
        let metadata = json!({
            "title": data.title,
            "description": data.description,
        });
        let meta_vec = serde_json::to_vec_pretty(&metadata).unwrap();
        let mut meta_bytes = vec![0; meta_vec.len()];
        meta_bytes.clone_from_slice(&meta_vec);
        let meta_oid = self.repository.blob(&meta_bytes)?;

        let mut treebuilder = match branch {
            Some(&ref branch) => self.repository.treebuilder(Some(&branch.tree().unwrap())),
            None => self.repository.treebuilder(None),
        }?;

        treebuilder
            .insert("shader.txt", source_oid, 0o100644)
            .unwrap();
        treebuilder
            .insert("metadata.json", meta_oid, 0o100644)
            .unwrap();
        self.commit_treebuilder(branch, &treebuilder, message)
    }

    pub fn add(&self, data: &ShaderData, message: &str) -> Result<(String, String), Error> {
        let uuid = format!("{}", uuid::Uuid::new_v4().to_hyphenated());

        let commit_oid = self.create_commit(None, data, message)?;
        let commit = self.repository.find_commit(commit_oid)?;
        self.repository
            .branch(&vec![BRANCH_PREFIX, &uuid].join(""), &commit, true)?;
        Ok((uuid, format!("{}", commit_oid)))
    }

    pub fn update(
        &self,
        name: &str,
        data: &ShaderData,
        revision: &str,
        message: &str,
    ) -> Result<String, Error> {
        let branchobj = self
            .repository
            .revparse_single(&vec![BRANCH_PREFIX, name].join(""))?;
        let branch = branchobj.as_commit().unwrap();
        if branch.id() != Oid::from_str(revision)? {
            return Err(Error::from_str("Shader was modified concurrently."));
        }

        let commit_oid = self.create_commit(Some(branch), data, message)?;
        Ok(format!("{}", commit_oid))
    }

    pub fn remove(&self, name: &str) -> Result<(), Error> {
        let mut branch = self
            .repository
            .find_branch(&vec![BRANCH_PREFIX, name].join(""), BranchType::Local)?;
        branch.delete().ok();
        Ok(())
    }
}
