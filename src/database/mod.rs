use git2;
use git2::{Repository, ObjectType, Commit, Signature, Oid, TreeBuilder};
use log;
use std;
use std::io::{Error, ErrorKind};

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

    pub fn list(&self) -> Result<Vec<String>, Box<std::error::Error>> {
        let obj = self.repository.revparse_single("master")?;

        match obj.kind() {
            None => Ok(Vec::new()),
            Some(ObjectType::Tree) => Ok(Vec::new()),
            Some(ObjectType::Any) => Ok(Vec::new()),
            Some(ObjectType::Commit) => Ok(obj.as_commit().unwrap().tree().unwrap().iter().map(|entry| {
                                format!("{}", entry.id())
                            }).collect()),
            Some(ObjectType::Blob) => Ok(Vec::new()),
            Some(ObjectType::Tag) => Ok(Vec::new()),
        }
    }

    pub fn read(&self, id: &str) -> Result<String, Box<std::error::Error>> {
        let obj = self.repository.revparse_single(id)?;

        match obj.kind() {
            None => Err(Box::new(Error::new(ErrorKind::NotFound, "none found"))),
            Some(ObjectType::Tree) => Err(Box::new(Error::new(ErrorKind::NotFound, "found tree"))),
            Some(ObjectType::Any) => Err(Box::new(Error::new(ErrorKind::NotFound, "found any"))),
            Some(ObjectType::Commit) => Err(Box::new(Error::new(ErrorKind::NotFound, "found commit"))),
            Some(ObjectType::Tag) => Err(Box::new(Error::new(ErrorKind::NotFound, "found tag"))),
            Some(ObjectType::Blob) => {
                match String::from_utf8(Vec::from(obj.as_blob().unwrap().content())) {
                    Ok(str) => Ok(str),
                    Err(utf8err) => Err(Box::new(utf8err.utf8_error()))
                }
            },
        }
    }

    fn commit_treebuilder(&self, parent: &Commit, treebuilder: &TreeBuilder, message: &str) -> Result<String, git2::Error> {
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

    pub fn add(&mut self, name: &str, content: &str, message: &str) -> Result<String, git2::Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let bytes = content.as_bytes();
        let oid = self.repository.blob(bytes)?;
        let mut treebuilder = self.repository.treebuilder(Some(&master.tree().unwrap()))?;
        treebuilder.insert(name, oid, 0o100644).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)
    }

    pub fn remove(&mut self, id: &str, message: &str) -> Result<String, git2::Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let master_tree = master.tree()?;
        let entry = master_tree.get_id(Oid::from_str(id).unwrap()).unwrap();

        let mut treebuilder = self.repository.treebuilder(Some(&master_tree))?;
        treebuilder.remove(entry.name().unwrap()).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)
    }
}
