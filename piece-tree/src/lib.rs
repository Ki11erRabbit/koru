use std::cell::RefCell;
use std::rc::Rc;

pub enum Color {
    Black,
    Red,
}

pub struct TreeNode {
    parent: Option<Rc<RefCell<TreeNode>>>,
    left: Option<Rc<RefCell<TreeNode>>>,
    right: Option<Rc<RefCell<TreeNode>>>,
    color: Color,
    piece: Piece,
    /// size of the left subtree
    size_left: usize,
    // newline counts in the left subtree
    newline_left: usize,
}

impl TreeNode {
    pub fn new(piece: Piece, color: Color) -> Rc<RefCell<TreeNode>> {
        let node = TreeNode {
            piece,
            color,
            parent: None,
            left: None,
            right: None,
            size_left: 0,
            newline_left: 0,
        };
        
        let node = Rc::new(RefCell::new(node));
        let copy = node.clone();
        let mut guard = node.borrow_mut();
        guard.parent = Some(copy.clone());
        guard.left = Some(copy.clone());
        guard.right = Some(copy);
        
        node
    }
    
    fn sentinel() -> Rc<RefCell<TreeNode>> {
        let node = Self::new(Piece::new(0,0,0,0,0), Color::Black);
        node
    }
}

pub struct BufferCursor {
    line: usize,
    column: usize,
}

pub struct Piece {
    buffer_index: usize,
    start: BufferCursor,
    end: BufferCursor,
    length: usize,
    newline_count: usize,
}

impl Piece {
    pub fn new(
        buffer_index: usize,
        start: BufferCursor,
        end: BufferCursor,
        length: usize,
        newline_count: usize,
    ) -> Self {
        Self {
            buffer_index,
            start,
            end,
            length,
            newline_count,
        }
    }
}

pub struct StringBuffer {
    buffer: String,
    line_starts: Vec<usize>,
}

impl StringBuffer {
    pub fn new(buffer: &str) -> Self {
        let mut line_starts = Vec::new();
        let mut next_char_is_start = true;
        for (i, c) in buffer.chars().enumerate() {
            if next_char_is_start {
                line_starts.push(i);
                next_char_is_start = false;
            }
            if c == '\n' {
                next_char_is_start = true;
            }
        }
        StringBuffer {
            buffer: buffer.to_string(),
            line_starts,
        }
    }
    
}


pub struct PieceTree {
    root: Option<Rc<RefCell<TreeNode>>>,
    buffers: Vec<StringBuffer>,
    line_count: usize,
    length: usize,
}

impl PieceTree {
    pub fn new(buffers: Vec<StringBuffer>) -> Self {
        let mut piece_tree = PieceTree {
            buffers: vec![StringBuffer::new("")],
            root: None,
            line_count: 0,
            length: 0,
        };
        
        let mut last_node = None;
        for (i, buffer) in buffers.into_iter().enumerate() {
            if buffer.buffer.len() > 0 {
                let char_count = buffer.buffer.chars().count();
                let piece = Piece::new(
                    i + 1,
                    BufferCursor { line: 0, column: 0 },
                    BufferCursor { 
                        line: buffer.line_starts.len() - 1, 
                        column: char_count - *buffer.line_starts.last().unwrap()
                    },
                    char_count,
                    buffer.line_starts.len() - 1,
                );
                piece_tree.buffers.push(buffer);
                last_node = piece_tree.rb_insert_right(last_node, piece);
            }
        }
        piece_tree
    }
    
    fn rb_insert_right(&mut self, node: Option<Rc<RefCell<TreeNode>>>, piece: Piece) -> Option<Rc<RefCell<TreeNode>>> {
        let z = TreeNode::new(piece, Color::Red);
        let sentinel = TreeNode::sentinel();
        {
            let mut z_guard = z.borrow_mut();

            z_guard.left = sentinel.clone();
            z_guard.right = sentinel.clone();
            z_guard.parent = sentinel.clone();
            
        }
        
        let root = self.root.clone();
        
        if root.is_some() && root.clone().unwrap() == sentinel {
            self.root = Some(z.clone());
            z.borrow_mut().color = Color::Black;
        } else if node.is_some() && node.clone().unwrap() == sentinel {
            let Some(node) = node else {
                unreachable!("We just checked for non None");
            };
            node.borrow_mut().right = Some(z.clone());
            z.borrow_mut().parent = node.clone();
        } else {
            let next_node = self.leftest(node.unwrap().borrow().right.clone());
            if let Some(next_node) = next_node {
                next_node.borrow_mut().left = Some(z.clone());
                z.borrow_mut().parent = Some(next_node);
            }
        }
        
        self.fix_insert(Some(z.clone()));
        
        Some(z)
    }
}