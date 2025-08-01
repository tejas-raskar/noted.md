use anyhow::Result;
use comrak::{
    Arena, ComrakOptions,
    nodes::{AstNode, ListType, NodeValue},
    parse_document,
};
use notion_client::objects::{
    block::{
        Block, BlockType, BulletedListItemValue, EquationValue, HeadingsValue,
        NumberedListItemValue, ParagraphValue,
    },
    rich_text::{self, RichText},
};

pub struct Converter<'a> {
    _arena: &'a Arena<AstNode<'a>>,
}

impl<'a> Converter<'a> {
    pub fn run(markdown: &str, arena: &'a Arena<AstNode<'a>>) -> Result<Vec<Block>, anyhow::Error> {
        let mut options = ComrakOptions::default();
        options.extension.math_dollars = true;
        let root = parse_document(arena, markdown, &options);
        let mut converter = Self { _arena: arena };
        let blocks = converter.render_nodes(root.children())?;

        Ok(blocks)
    }
    fn render_nodes(
        &mut self,
        nodes: impl Iterator<Item = &'a AstNode<'a>>,
    ) -> Result<Vec<Block>, anyhow::Error> {
        let mut blocks = Vec::new();
        for node in nodes {
            blocks.extend(self.render_node(node)?);
        }
        Ok(blocks)
    }

    fn render_node(&mut self, node: &'a AstNode<'a>) -> Result<Vec<Block>> {
        match &node.data.borrow().value {
            NodeValue::Heading(heading) => Ok(vec![self.render_heading(node, heading)?]),
            NodeValue::Paragraph => {
                let mut children = node.children();
                if let (Some(child), None) = (children.next(), children.next()) {
                    if let NodeValue::Math(_) = &child.data.borrow().value {
                        return Ok(vec![self.render_math(child)?]);
                    }
                }
                Ok(vec![self.render_paragraph(node)?])
            }
            NodeValue::List(list) => match list.list_type {
                ListType::Bullet => self.render_bullet_list(node),
                ListType::Ordered => self.render_numbered_list(node),
            },
            _ => Ok(Vec::new()),
        }
    }

    fn render_bullet_list(&mut self, node: &'a AstNode<'a>) -> Result<Vec<Block>> {
        let mut items = Vec::new();
        for child in node.children() {
            let block = self.render_bulleted_list_item(child)?;
            items.push(block);
        }
        Ok(items)
    }

    fn render_numbered_list(&mut self, node: &'a AstNode<'a>) -> Result<Vec<Block>> {
        let mut items = Vec::new();
        for child in node.children() {
            let block = self.render_numbered_list_item(child)?;
            items.push(block);
        }
        Ok(items)
    }

    fn render_numbered_list_item(&mut self, node: &'a AstNode<'a>) -> Result<Block> {
        let mut rich_text = Vec::new();

        if let Some(paragraph) = node
            .children()
            .find(|child| matches!(child.data.borrow().value, NodeValue::Paragraph))
        {
            rich_text = self.render_rich_text(paragraph)?;
        }

        let value = NumberedListItemValue {
            rich_text,
            color: notion_client::objects::block::TextColor::Default,
            children: None,
        };

        Ok(Block {
            block_type: BlockType::NumberedListItem {
                numbered_list_item: value,
            },
            ..Default::default()
        })
    }

    fn render_bulleted_list_item(&mut self, node: &'a AstNode<'a>) -> Result<Block> {
        let mut rich_text = Vec::new();

        if let Some(paragraph) = node
            .children()
            .find(|child| matches!(child.data.borrow().value, NodeValue::Paragraph))
        {
            rich_text = self.render_rich_text(paragraph)?;
        }

        let value = BulletedListItemValue {
            rich_text,
            color: notion_client::objects::block::TextColor::Default,
            children: None,
        };

        Ok(Block {
            block_type: BlockType::BulletedListItem {
                bulleted_list_item: value,
            },
            ..Default::default()
        })
    }

    fn render_math(&mut self, node: &'a AstNode<'a>) -> Result<Block> {
        if let NodeValue::Math(math) = &node.data.borrow().value {
            let expression = math.literal.clone();
            let value = EquationValue { expression };
            let block_type = BlockType::Equation { equation: value };
            Ok(Block {
                block_type,
                ..Default::default()
            })
        } else {
            Err(anyhow::anyhow!(
                "Node passed to render_math was not a Math node"
            ))
        }
    }

    fn render_paragraph(&mut self, node: &'a AstNode<'a>) -> Result<Block> {
        let rich_text = self.render_rich_text(node)?;
        let value = ParagraphValue {
            rich_text,
            ..Default::default()
        };
        let block_type = BlockType::Paragraph { paragraph: value };
        Ok(Block {
            block_type,
            ..Default::default()
        })
    }

    fn render_heading(
        &mut self,
        node: &'a AstNode<'a>,
        heading: &comrak::nodes::NodeHeading,
    ) -> Result<Block> {
        let rich_text = self.render_rich_text(node)?;

        let value = HeadingsValue {
            rich_text,
            ..Default::default()
        };
        let block_type = match &heading.level {
            1 => BlockType::Heading1 { heading_1: value },
            2 => BlockType::Heading2 { heading_2: value },
            _ => BlockType::Heading3 { heading_3: value },
        };

        Ok(Block {
            block_type,
            ..Default::default()
        })
    }

    fn render_rich_text(
        &mut self,
        node: &'a AstNode<'a>,
    ) -> Result<Vec<notion_client::objects::rich_text::RichText>> {
        let mut rich_text_nodes = Vec::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(text) => {
                    rich_text_nodes.push(notion_client::objects::rich_text::RichText::Text {
                        text: notion_client::objects::rich_text::Text {
                            content: text.clone(),
                            link: None,
                        },
                        annotations: Default::default(),
                        plain_text: Some(text.clone()),
                        href: None,
                    });
                }
                NodeValue::Math(math) => {
                    let latex = math.literal.clone();
                    rich_text_nodes.push(RichText::Equation {
                        equation: rich_text::Equation {
                            expression: latex.clone(),
                        },
                        annotations: Default::default(),
                        plain_text: latex.to_string(),
                        href: None,
                    })
                }
                _ => {}
            }
        }
        Ok(rich_text_nodes)
    }
}
