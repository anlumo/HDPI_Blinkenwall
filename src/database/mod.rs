use git2::{Repository, ObjectType, Commit, Signature, Oid, TreeBuilder, Error};

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

    pub fn read(&self, id: &str) -> Result<String, Error> {
        match self.repository.revparse_single(id) {
            Err(error) => Err(error),
            Ok(obj) => {
                match obj.kind() {
                    None => Err(Error::from_str("none found")),
                    Some(ObjectType::Tree) => Err(Error::from_str("found tree")),
                    Some(ObjectType::Any) => Err(Error::from_str("found any")),
                    Some(ObjectType::Commit) => Err(Error::from_str("found commit")),
                    Some(ObjectType::Tag) => Err(Error::from_str("found tag")),
                    Some(ObjectType::Blob) => {
                        match String::from_utf8(Vec::from(obj.as_blob().unwrap().content())) {
                            Ok(str) => Ok(str),
                            Err(utf8err) => Err(Error::from_str(&format!("{}", utf8err.utf8_error())))
                        }
                    },
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
        treebuilder.insert(format!("{}", oid), oid, 0o100644).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)
    }

    pub fn remove(&mut self, id: &str, message: &str) -> Result<String, Error> {
        let masterobj = self.repository.revparse_single("master")?;
        let master = masterobj.as_commit().unwrap();
        let master_tree = master.tree()?;
        let entry = master_tree.get_id(Oid::from_str(id).unwrap()).unwrap();

        let mut treebuilder = self.repository.treebuilder(Some(&master_tree))?;
        treebuilder.remove(entry.name().unwrap()).unwrap();
        self.commit_treebuilder(&master, &treebuilder, &message)
    }
}
