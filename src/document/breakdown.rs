use serde::{Deserialize, Serialize};

use crate::{types::StandaloneValue, Document};

use super::NodeBreakdown;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentBreakdown {
    pub standalone_version: Option<StandaloneValue>,
    pub version: String,
    pub root_elements: Vec<NodeBreakdown>,
}

impl Document {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn breakdown(&self) -> DocumentBreakdown {
        let root_nodes = self.root_nodes();
        let mut root_elements = Vec::with_capacity(root_nodes.len());

        for root in root_nodes {
            let root_breakdown = root.breakdown(self);
            root_elements.push(root_breakdown);
        }
        DocumentBreakdown {
            standalone_version: self.standalone,
            version: self.version.clone(),
            root_elements,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::tests::setup_logger;
    use crate::Document;

    #[test]
    fn breakdown() {
        setup_logger();
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <project>
            <name>edit-xml</name>
            <version>0.1.0</version>
            <description>XML editing library</description>
            <license>MIT</license>
        </project>
        "#;
        let doc = Document::parse_str(xml).unwrap();
        let mut breakdown: Vec<NodeBreakdown> = doc.breakdown().root_elements;
        assert_eq!(breakdown.len(), 1, "Expected 1 root element");
        let project = breakdown.pop().unwrap();
        println!("{:#?}", project);
        let NodeBreakdown::Element(project) = project else {
            panic!("Expected root element to be an Element");
        };
        assert_eq!(project.name, "project");
        assert_eq!(project.children.len(), 4);
        let name = project.children.first().unwrap();
        let NodeBreakdown::Element(name) = name else {
            panic!("Expected child to be an Element");
        };
        println!("{:?}", name);
    }
}
