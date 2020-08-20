use anyhow::Result;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct FileManipulator {
    pub content: String,
    pub file_path: String,
}

impl FileManipulator {
    pub fn new(file_path: String, default_content: &str) -> Result<Self> {
        let mut file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)
        {
            Ok(f) => f,
            Err(_e) => {
                let mut file = std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&file_path)?;
                file.write_all(default_content.as_bytes())?;
                std::fs::OpenOptions::new().read(true).open(&file_path)?
            }
        };
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(FileManipulator {
            content: content,
            file_path: file_path,
        })
    }
    pub fn insert_between(
        &mut self,
        between: &str,
        content: &str,
        after: Option<&str>,
    ) -> Result<()> {
        let re = regex::Regex::new(between).expect("Invalid regex");
        let splitted: Vec<&str> = re.split(&self.content).map(|s| s).collect();
        if splitted.len() != 3 {
            if let Some(after) = after {
                match self.insert_after(after, content, between) {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(anyhow::anyhow!(e)),
                };
            }
            return Err(anyhow::anyhow!(
                "Between pattern had {} partitions. Could not determine between. Maybe use insert_after",
                splitted.len()
            ));
        }
        let mut new_content = String::new();
        for (i, part) in splitted.iter().enumerate() {
            if i == 1 {
                let content = format!("{}{}{}{}", between, content, "\n", between);
                new_content.push_str(&content);
            } else {
                new_content.push_str(&part);
            }
        }
        self.content = new_content;
        Ok(())
    }
    pub fn insert_after(
        &mut self,
        after: &str,
        content: &str,
        between_pattern: &str,
    ) -> Result<()> {
        let re = regex::Regex::new(&after).expect("Invalid regex");
        let splitted: Vec<&str> = re.split(&self.content).map(|s| s).collect();
        if splitted.len() != 2 {
            return Err(anyhow::anyhow!(
                "After {} pattern had {} partition(s). Could not determine after what in {}.",
                after,
                splitted.len(),
                self.file_path,
            ));
        }
        let mut new_content = String::new();
        for (i, part) in splitted.iter().enumerate() {
            if i == 1 {
                let content = format!(
                    "{}{}{}{}{}{}{}",
                    after, "\n", &between_pattern, content, "\n", &between_pattern, part
                );
                new_content.push_str(&content);
            } else {
                new_content.push_str(&part);
            }
        }
        self.content = new_content;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .expect("Required that file exist to save");
        file.write_all(self.content.as_bytes())
            .expect("Could not write to file");
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_must_use, unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn read_empty_file() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        let some_file = FileManipulator::new(test_file.to_string(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");
        assert!(some_file.content.len() > 20);
        assert!(std::fs::remove_file(&test_file).is_ok());
    }

    #[test]
    fn read_file() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        std::fs::remove_file(&test_file);

        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&test_file)
            .expect("Could not create test_file");
        file.write_all(SOME_JS_CONTENT_WITH_CONSTRUCTOR.as_bytes())
            .expect("Could not write to test_file");
        let some_file = FileManipulator::new(test_file.to_string(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");
        assert!(some_file.content.contains("signer"));
        assert!(std::fs::remove_file(&test_file).is_ok());
    }
    #[test]
    fn swap_component_constructor() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&test_file)
            .expect("Could not create test_file");
        file.write_all(SOME_JS_CONTENT_WITH_CONSTRUCTOR.as_bytes())
            .expect("Could not write to test_file");
        let mut some_file = FileManipulator::new(test_file.to_string(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");

        assert!(some_file.content.contains("SimpleStorageConstructor"));
        let some_multiline_content = r#"
        SOME
        MULTILINE
        CONTENT
        "#;
        some_file
            .insert_between("// SimpleStorageConstructor", some_multiline_content, None)
            .expect("Could not insert between SimpleStorageConstructor");
        assert!(some_file.content.contains(some_multiline_content));
        some_file.save().expect("Could not save file");
        let mut file_contents = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&some_file.file_path)
            .expect("Could not open saved file")
            .read_to_string(&mut file_contents)
            .expect("Could not write file contents");
        assert!(file_contents.contains(some_multiline_content));
        assert!(std::fs::remove_file(test_file).is_ok());
    }

    #[test]
    fn try_insert_between_then_insert_after() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        std::fs::remove_file(&test_file);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&test_file)
            .expect("Could not create test_file");
        file.write_all(SOME_JS_CONTENT_WITH_CONSTRUCTOR.as_bytes())
            .expect("Could not write to test_file");
        let mut some_file = FileManipulator::new(test_file.clone(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");

        assert!(some_file.content.contains("SimpleStorageConstructor"));
        let some_multiline_content = r#"
        SOME
        MULTILINE
        CONTENT
        "#;
        some_file
            .insert_between("// JIBBERISH", some_multiline_content, Some("// BODY"))
            .expect("Could not insert between SimpleStorageConstructor");
        assert!(some_file.content.contains(some_multiline_content));
        some_file.save().expect("Could not save file");
        let mut file_contents = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&some_file.file_path)
            .expect("Could not open saved file")
            .read_to_string(&mut file_contents)
            .expect("Could not write file contents");
        assert!(file_contents.contains(some_multiline_content));
        assert!(std::fs::remove_file(&test_file).is_ok());
    }

    #[test]
    fn try_insert_many() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        std::fs::remove_file(&test_file);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&test_file)
            .expect("Could not create test_file");
        file.write_all(SOME_JS_CONTENT_CLASS.as_bytes())
            .expect("Could not write to test_file");
        let mut some_file = FileManipulator::new(test_file.clone(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");

        assert!(some_file.content.contains("// IMPORT"));

        let imports = r#"
        import { CONTRACT_OBJECT } from "./ethereum/CONTRACT_OBJECT";
        import { CONTRACT_OBJECTFactory } from "./ethereum/CONTRACT_OBJECTFactory";
        "#;
        some_file
            .insert_between("// SimpleStorageImport", &imports, Some("// IMPORT"))
            .expect("imports");

        let constructor = r#"
        const wallet = ethers.Wallet.fromMnemonic("INPUT_MNEMONIC");
        const provider = new ethers.providers.JsonRpcProvider({
            url: "INPUT_NODE_URL",
        });
        const signer = wallet.connect(provider);
        this.CONTRACT_OBJECT = CONTRACT_OBJECTFactory.connect(
            "INPUT_ADDRESS",
            signer
        );
        "#;
        some_file
            .insert_between(
                "// SimpleStorageConstructor",
                &constructor,
                Some("// CONSTRUCTOR"),
            )
            .expect("Could not insert constructor");

        some_file.save().expect("Could not save file");
        let mut file_contents = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&some_file.file_path)
            .expect("Could not open saved file")
            .read_to_string(&mut file_contents)
            .expect("Could not write file contents");
        assert!(file_contents.contains(constructor));
        assert!(file_contents.contains(imports));
        assert!(std::fs::remove_file(&test_file).is_ok());
    }

    #[test]
    fn try_insert_many_after_then_between() {
        let rng: u32 = rand::random();
        let test_file = format!("test-{}.ts", rng);
        std::fs::remove_file(&test_file);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&test_file)
            .expect("Could not create test_file");
        file.write_all(SOME_JS_CONTENT_CLASS.as_bytes())
            .expect("Could not write to test_file");
        let mut some_file = FileManipulator::new(test_file.clone(), SOME_JS_CONTENT_CLASS)
            .expect("Could not create empty file");

        assert!(some_file.content.contains("// IMPORT"));

        // Insert import
        let imports = r#"
        import { CONTRACT_OBJECT } from "./ethereum/CONTRACT_OBJECT";
        import { CONTRACT_OBJECTFactory } from "./ethereum/CONTRACT_OBJECTFactory";
        "#;
        some_file
            .insert_between("// SimpleStorageImport", &imports, Some("// IMPORT"))
            .expect("imports");

        // insert constructor
        let constructor = r#"
        const wallet = ethers.Wallet.fromMnemonic("INPUT_MNEMONIC");
        const provider = new ethers.providers.JsonRpcProvider({
            url: "INPUT_NODE_URL",
        });
        const signer = wallet.connect(provider);
        this.CONTRACT_OBJECT = CONTRACT_OBJECTFactory.connect(
            "INPUT_ADDRESS",
            signer
        );
        "#;
        some_file
            .insert_between(
                "// SimpleStorageConstructor",
                &constructor,
                Some("// CONSTRUCTOR"),
            )
            .expect("Could not insert constructor");

        // Insert updated constructor
        let constructor_updated = r#"
        const updated = ethers.Wallet.fromMnemonic("INPUT_MNEMONIC");
        const provider = new ethers.providers.JsonRpcProvider({
            url: "INPUT_NODE_URL",
        });
        const signer = wallet.connect(provider);
        this.CONTRACT_OBJECT = CONTRACT_OBJECTFactory.connect(
            "INPUT_ADDRESS",
            signer
        );
        "#;
        some_file
            .insert_between(
                "// SimpleStorageConstructor",
                &constructor_updated,
                Some("// CONSTRUCTOR"),
            )
            .expect("Could not insert updated constructor");

        // insert updated import
        let imports_updated = r#"
        import { CONTRACT_UPDATED } from "./ethereum/CONTRACT_OBJECT";
        import { CONTRACT_OBJECTFactory } from "./ethereum/CONTRACT_OBJECTFactory";
        "#;
        some_file
            .insert_between(
                "// SimpleStorageImport",
                &imports_updated,
                Some("// IMPORT"),
            )
            .expect("imports");

        some_file.save().expect("Could not save file");
        let mut file_contents = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&some_file.file_path)
            .expect("Could not open saved file")
            .read_to_string(&mut file_contents)
            .expect("Could not write file contents");
        assert!(file_contents.contains(constructor_updated));
        assert!(file_contents.contains(imports_updated));
        assert!(std::fs::remove_file(&test_file).is_ok());
    }

    static SOME_JS_CONTENT_CLASS: &str = r#"
// IMPORT
import { ethers } from "ethers";

// CLASS
export class SomeParent {

    constructor() {
      // CONSTRUCTOR
    }

    // BODY
}     
"#;

    static SOME_JS_CONTENT_WITH_CONSTRUCTOR: &str = r#"
// IMPORT
import { ethers } from "ethers";
import { SimpleStorage } from "./ethereum/SimpleStorage";
import { SimpleStorageFactory } from "./ethereum/SimpleStorageFactory";
// CLASS
class Some {
  SimpleStorage: SimpleStorage;
  constructor() {
    // CONSTRUCTOR
    // SimpleStorageConstructor
    const wallet = ethers.Wallet.fromMnemonic(
      "SOME SECRET"
    );
    const provider = new ethers.providers.JsonRpcProvider({
      url: "SOME_NODE",
    });
    const signer = wallet.connect(provider);
    this.SimpleStorage = SimpleStorageFactory.connect(
      "0x78Bf5369de74281FecC250D75419d0DAF1dE9f7d",
      signer
    );
    // SimpleStorageConstructor
  }

  // BODY
}
        "#;
}
