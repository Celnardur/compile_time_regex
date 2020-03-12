use std::vec::Vec;
use crate::*;
use crate::pointer::Pointer;
use std::{rc::Rc, cell::RefCell};

/*
 * Work through of a binary tree stored as a vector
 *
 * Example Tree Depth of 4
 *         ----------00----------
 *         |                    |
 *    -----10-----         -----11-----
 *    |          |         |          |
 *    20         21        22         23
 *   /  \       /  \      /  \       /  \
 *  30  31     32  33    34  35     36  37
 *
 * The Trees elements are stored in pre-order in a vector.
 * This means that the pre-order is the forward iterator and 
 * post-order is the reverse iterator.
 *
 * Stored as:
 * vec![00, 10, 20, 30, 31, 21, 32, 33, 11, 22, 34, 35, 23, 36, 37]
 *      0   1   2   3   4   5   6   7   8   9   10  11  12  13  14
 *
 * length = 1 + 2 + 4 + 8 
 *        = 2^0 + 2^1 + 2^2 + 2^3 
 *        = 1 << depth - 1
 *        = 15
 * 
 * this also means the max depth is 64 because reaches that max length of vector
 * This number is actually unreasonably small so this method of storing a tree is
 * not practical.
 */

/**
 * You can create a new BinaryTree with macros like this:
 * ```rust
 * let tree = bNode!( "head", 
 *     bNode!( "left",
 *         bNode!("left-left"),
 *         bNode!("left-right"),
 *      ),
 *      bNode!("right", bNode!(22), None));
 * ```
 */


pub struct BinaryTree<T> {
    value: T,
    left: Option<Pointer<BinaryTree<T>>>,
    right: Option<Pointer<BinaryTree<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn new(value: T) -> Self {
        BinaryTree {
            value,
            None,
            None,
        }
    }

    pub right(&self) -> Self {
        match self.right() {
            Some(right) => Some(ptrcp!(right)),
            None => None,
        }
    }
}

