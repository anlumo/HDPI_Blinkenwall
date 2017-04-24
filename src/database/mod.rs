use git2::{Repository, ObjectType, Commit, Signature, TreeBuilder, Error};
use uuid;

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
        }).collect())
    }

    pub fn read(&self, name: &str) -> Result<String, Error> {
        let tree = self.repository.revparse_single("master")?.as_commit().unwrap().tree()?;
        let entry = tree.get_name(name);
        match entry {
            None => Err(Error::from_str("not found")),
            Some(entry) => {
                let obj = entry.to_object(&self.repository)?;
                let file = obj.as_blob().unwrap();
                match String::from_utf8(Vec::from(file.content())) {
                    Ok(str) => Ok(str),
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

    pub fn add(&mut self, content: &str, message: &str) -> Result<String, Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let bytes = content.as_bytes();
        let oid = self.repository.blob(bytes)?;
        let mut treebuilder = self.repository.treebuilder(Some(&master.tree().unwrap()))?;

        let uuid = uuid::Uuid::new_v5(&uuid::NAMESPACE_URL, "blinkenwall");
        let hyphenated = format!("{}", uuid.hyphenated());

        treebuilder.insert(&hyphenated, oid, 0o100644).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)?;
        Ok(hyphenated)
    }

    pub fn remove(&mut self, name: &str, message: &str) -> Result<String, Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let master_tree = master.tree()?;

        let mut treebuilder = self.repository.treebuilder(Some(&master_tree))?;
        treebuilder.remove(name)?;
        self.commit_treebuilder(&master, &treebuilder, &message)
    }
}
