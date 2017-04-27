use git2::{Repository, ObjectType, Commit, Signature, TreeBuilder, Error};
use uuid;
use server::ShaderData;
use serde_json;

pub struct Database {
    repository: Repository
}

impl Database {
    pub fn new(path: &str) -> Database {
        let repo = Repository::open(path).unwrap();

        Database {
            repository: repo
        }
    }

    pub fn list(&self) -> Result<Vec<String>, Error> {
        let obj = self.repository.revparse_single("master")?;

        let tree = obj.as_commit().unwrap().tree()?;
        Ok(tree.iter().map(|entry| {
            tree.get_id(entry.id()).unwrap().name().unwrap().to_string()
        }).filter(|entry| { !entry.ends_with(".meta") }).collect())
    }

    pub fn read(&self, name: &str) -> Result<ShaderData, Error> {
        let tree = self.repository.revparse_single("master")?.as_commit().unwrap().tree()?;
        let entry = tree.get_name(name);
        match entry {
            None => Err(Error::from_str("not found")),
            Some(entry) => {
                let obj = entry.to_object(&self.repository)?;
                let file = obj.as_blob().unwrap();
                match String::from_utf8(Vec::from(file.content())) {
                    Ok(source) => {
                        match tree.get_name(&format!("{}.meta", name)) {
                            None => Err(Error::from_str("not found")),
                            Some(meta) => {
                                match serde_json::from_str(&String::from_utf8(Vec::from(meta.to_object(&self.repository)?.as_blob().unwrap().content())).unwrap()) {
                                    Ok(obj) => {
                                        let obj : serde_json::Value = obj;
                                        if let (&serde_json::Value::String(ref title), &serde_json::Value::String(ref description)) = (&obj["title"], &obj["description"]) {
                                            Ok(ShaderData {
                                                title: title.clone(),
                                                description: description.clone(),
                                                source: source,
                                            })
                                        } else {
                                            Err(Error::from_str("invalid format"))
                                        }
                                    },
                                    Err(error) => Err(Error::from_str(&format!("{}", error)))
                                }
                            }
                        }
                    },
                    Err(utf8err) => Err(Error::from_str(&format!("{}", utf8err.utf8_error())))
                }
            }
        }
    }

    fn commit_treebuilder(&self, parent: &Commit, treebuilder: &TreeBuilder, message: &str) -> Result<String, Error> {
        let treeoid = treebuilder.write().unwrap();
        let treeobj = self.repository.find_object(treeoid, Some(ObjectType::Tree)).unwrap();
        let tree = treeobj.as_tree().unwrap();

        let signature = Signature::now("Blinkenwall", "blinkenwall@monitzer.com").unwrap();
        match self.repository.commit(Some("HEAD"), &signature, &signature,
            message, &tree, &[&parent]) {
            Ok(oid) => {
                info!("Commit {} for {}", oid, message);
                Ok(format!("{}", oid))
            },
            Err(error) => Err(error)
        }
    }

    pub fn add(&mut self, data: &ShaderData, message: &str) -> Result<String, Error> {
        // TODO: lock repository
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
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

        let mut treebuilder = self.repository.treebuilder(Some(&master.tree().unwrap()))?;

        let uuid = uuid::Uuid::new_v5(&uuid::NAMESPACE_URL, "blinkenwall");
        let hyphenated = format!("{}", uuid.hyphenated());
        let metaname = format!("{}.meta", uuid.hyphenated());

        treebuilder.insert(&hyphenated, source_oid, 0o100644).unwrap();
        treebuilder.insert(&metaname, meta_oid, 0o100644).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)?;
        Ok(hyphenated)
    }

    pub fn remove(&mut self, name: &str, message: &str) -> Result<String, Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let master_tree = master.tree()?;

        let mut treebuilder = self.repository.treebuilder(Some(&master_tree))?;
        treebuilder.remove(name)?;
        treebuilder.remove(format!("{}.meta", name))?;
        self.commit_treebuilder(&master, &treebuilder, &message)
    }
}
