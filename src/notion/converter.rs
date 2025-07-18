use anyhow::Result;
use comrak::{
    Arena, ComrakOptions,
    nodes::{AstNode, NodeValue},
    parse_document,
};
use notion_client::objects::block::{
    Block, BlockType, BulletedListItemValue, HeadingsValue, ParagraphValue,
};

pub struct Converter<'a> {
    arena: &'a Arena<AstNode<'a>>,
}

impl<'a> Converter<'a> {
    pub fn run(markdown: &str, arena: &'a Arena<AstNode<'a>>) -> Result<Vec<Block>, anyhow::Error> {
        let root = parse_document(arena, markdown, &ComrakOptions::default());
        let mut converter = Self { arena };
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
            NodeValue::Paragraph => Ok(vec![self.render_paragraph(node)?]),
            NodeValue::List(_list) => self.render_nodes(node.children()),
            NodeValue::Item(_item) => Ok(vec![self.render_bulleted_list_item(node)?]),
            _ => Ok(Vec::new()),
        }
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
            if let NodeValue::Text(text) = &child.data.borrow().value {
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
        }
        Ok(rich_text_nodes)
    }
}
