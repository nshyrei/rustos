use core::cmp;
use core::cmp::Ordering;
use MemoryAllocator;
use heap;

type NodeBox<T, M>         = heap::RC<AVLNode<T, M>, M>;
type OptNodeBox<T, M>  = Option<NodeBox<T, M>>;

fn height<T, M>(node : &OptNodeBox<T, M>) -> i64 where T : cmp::Ord, M : MemoryAllocator {
    node.as_ref()
        .map(|n| n.height())
        .unwrap_or(-1)
}

fn insert0<T, M>(mut node : NodeBox<T, M>, value : T, memory_allocator : &mut M) -> NodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator  {
    let cmp_result = node.value().cmp(&value);

    match cmp_result {
        Ordering::Less => {
            if let Some(left_node) = node.left.take() {
                let new_left = insert0(left_node, value, memory_allocator);
                node.set_left(Some(new_left));
            }
            else {
                node.set_left(Some(AVLNode::new(value, 0, memory_allocator)));
            }
        },
        Ordering::Greater => {
            if let Some(right_node) = node.right.take() {
                let new_right = insert0(right_node, value, memory_allocator);
                node.set_right(Some(new_right));
            }
            else {
                node.set_right(Some(AVLNode::new(value, 0, memory_allocator)));
            }
        },
        _ => return node
    }
        
    node.update_height();

    balance(node)
}


fn find0<'a, T, M>(node : &'a OptNodeBox<T, M>, x : &T) -> Option<&'a T> where T : cmp::Ord, M : MemoryAllocator {
    node.as_ref().and_then(|n| match n.value().cmp(x) {
            Ordering::Less    => find0(&n.left, x),
            Ordering::Greater => find0(&n.right, x),
            _                 => Some(&n.value)
    })
}

fn find_by0<T, F, P, C, M>(node : &OptNodeBox<T, M>, x : &C, selector : F, predicate : P) -> Option<heap::WeakBox<T>>
    where F : Fn(&T) -> C,
          P : Fn(&T, &C) -> bool,
          T : cmp::Ord,
          C : cmp::Ord,
        M : MemoryAllocator
{
    node.as_ref().and_then(|n| {
        
        if predicate(n.value(), x) {
            Some(heap::WeakBox::from_pointer(n.value()))
        }
        else { 
            match selector(n.value()).cmp(x)  {
                Ordering::Less => find_by0(&n.left(), x, selector, predicate),
                Ordering::Greater => find_by0(&n.right(), x, selector, predicate),
                _ => Some(heap::WeakBox::from_pointer(n.value()))
            }
        }
    })    
}

fn rotate_right<T, M>(mut x : NodeBox<T, M>) -> NodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    let mut y = x.take_left().expect("invalid avl");

    x.set_left(y.take_right());
    x.update_height();
        
    y.set_right(Some(heap::RC::clone(&x)));
    y.update_height();

    y
}

fn rotate_left<T, M>(mut x : NodeBox<T, M>) -> NodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    let mut y = x.take_right().expect("invalid avl");

    x.set_right(y.take_left());
    y.set_left(Some(heap::RC::clone(&x)));

    x.update_height();
    y.update_height();

    y
}

fn balance_factor_opt<T, M>(node : &OptNodeBox<T, M>) -> i64 where T : cmp::Ord, M : MemoryAllocator {
    node.as_ref()
        .map(|n| balance_factor(n))
        .unwrap_or(0)
}

fn balance_factor<T, M>(node : &AVLNode<T, M>) -> i64 where T : cmp::Ord, M : MemoryAllocator {
    height(&node.left) - height(&node.right)
}

fn balance<T, M>(mut node : NodeBox<T, M>) -> NodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    let balance_factor = balance_factor(&node);

    if balance_factor < -1 {
        if balance_factor_opt(&node.right) > 0 && node.has_right() {
            let r = node.take_right_unwrap();

            node.set_right(Some(rotate_right(r)));
        }

        node = rotate_left(node);
    }
    else if balance_factor > 1 {
        if balance_factor_opt(&node.left) < 0 && node.has_left() {
            let l = node.take_left_unwrap();

            node.set_left(Some(rotate_left(l)));
        }

        node = rotate_right(node);
    }

    node
}

fn min<T, M>(node : NodeBox<T, M>) -> NodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    node.left()
        .as_ref()
        .map(|rc| min(heap::RC::clone(rc)))
        .unwrap_or(node)
}

fn delete_min<T, M>(mut node : NodeBox<T, M>) -> OptNodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    match node.left.take() {
        Some(left) => {
            node.set_left(delete_min(left));
            node.update_height();

            Some(balance(node))
        },
        _ => node.right()
    }
}

fn delete<T, M>(mut node : NodeBox<T, M>, value : T) -> OptNodeBox<T, M> where T : cmp::Ord, M : MemoryAllocator {
    let cmp_result = node.value().cmp(&value);
            
    match cmp_result {
        Ordering::Less => {
            let new_left = delete(node.take_left_unwrap(), value);
            node.set_left(new_left);
        },
        Ordering::Greater => {
            let new_right = delete(node.take_right_unwrap(), value);
            node.set_right(new_right);
        },
        _ => {
            if node.left().is_none() {
                return node.right()
            }
            else if node.right().is_none() {
                return node.left()
            }
            else {
                let y = heap::RC::clone(&node);

                let cpy_right  = y.right().is_some();
                let cpy_left    = y.left().is_some();

                {
                    let new_node = min(y.right().unwrap());

                    node = (new_node);
                }

                let new_right = delete_min(y.right().unwrap());
                node.set_right(new_right);
                node.set_left(y.left());
            }
        }
    }
        
    node.update_height();
    Some(balance(node))
}

fn delete_by<T, Q, F, M>(mut node : NodeBox<T, M>, value : Q, f : F) -> OptNodeBox<T, M> where T : cmp::Ord, F : Fn(&T) -> Q, Q :cmp::Ord, M : MemoryAllocator {
    let comparable_value = f(node.value());
    let cmp_result = comparable_value.cmp(&value);

    match cmp_result {
        Ordering::Less => {
            let new_left = delete_by(node.take_left_unwrap(), value, f);
            node.set_left(new_left);
        },
        Ordering::Greater => {
            let new_right = delete_by(node.take_right_unwrap(), value, f);
            node.set_right(new_right);
        },
        _ => {
            if node.left().is_none() {
                return  node.right()
            }
            else if node.right().is_none() {
                return node.left()
            }
            else {
                let y = heap::RC::clone(&node);

                let cpy_right  = y.right().is_some();
                let cpy_left     = y.left().is_some();

                {
                    let new_node = min(y.right().unwrap());

                    node = new_node;
                }

                let new_right = delete_min(y.right().unwrap());
                node.set_right(new_right);
                node.set_left(y.left());
            }
        }
    }

    node.update_height();
    Some(balance(node))
}

fn is_BST<T, M>(node : &OptNodeBox<T, M>, min : Option<&T>, max : Option<&T>) -> bool where T : cmp::Ord, M : MemoryAllocator {
    node.as_ref()
        .map(|n| {
            let value = n.value();
            let min_check = min.is_none() || value <= min.unwrap();
            let max_check = max.is_none() || value >= max.unwrap();

            min_check && 
            max_check && 
            is_BST(&n.left(), min, Some(value)) &&
            is_BST(&n.right(), Some(value), max)
        })
        .unwrap_or(true)
}

fn is_AVL<T, M>(node : &OptNodeBox<T, M>) -> bool where T : cmp::Ord, M : MemoryAllocator {
    node.as_ref()
        .map(|n| {
            let balance_factor = balance_factor(n);
            let balance_factor_check = balance_factor >= -1 && balance_factor <= 1;

            balance_factor_check &&
            is_AVL(&n.left()) &&
            is_AVL(&n.right())
        })
        .unwrap_or(true)
}

#[repr(C)]
pub struct AVLTree<T, M> where T : cmp::Ord, M : MemoryAllocator {
    root : OptNodeBox<T, M>
}

impl<T, M> AVLTree<T, M> where T : cmp::Ord, M : MemoryAllocator {

    pub fn root_value(&self) -> Option<&T> {
        self.root.as_ref().map(|rc| rc.value())
    }

    pub fn root_node(&self)   {
        let arv = self.root.as_ref().map(|n| {
            n.value()
        });
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn new(root_value : T, memory_allocator : &mut M) -> Self {
        let mut result = AVLTree::new_empty();
        result.insert(root_value, memory_allocator);

        result
    }

    pub fn new_empty() -> Self {
        AVLTree {
            root : None
        }
    }

    pub fn find_by<F, P, C>(&mut self, x : &C, selector : F, predicate : P) -> Option<heap::WeakBox<T>>
        where F : Fn(&T) -> C,
              P : Fn(&T, &C) -> bool,
              T : cmp::Ord,
              C : cmp::Ord
    {

        find_by0(&mut self.root, x, selector, predicate)
        
    }

    pub fn find(&self, x : &T) -> Option<&T> {
        find0(&self.root, x)
    }

    pub fn height(&self) -> i64 {
        height(&self.root)
    }

    pub fn insert(&mut self, value : T, memory_allocator : &mut M) where M : MemoryAllocator {
        match self.root.take() {
            Some(node) => {
                self.root = Some(insert0(node, value, memory_allocator))
            },
            _ => {
                self.root = Some(AVLNode::new(value, 0, memory_allocator));
            }
        }
    }

    pub fn delete(&mut self, key : T) {
        match self.root.take() {
            Some(node) => self.root = delete(node, key),
            _ => self.root = None
        }
    }

    pub fn delete_by<Q, F>(&mut self, key : Q, f : F)  where F : Fn(&T) -> Q, Q :cmp::Ord {
        match self.root.take() {
            Some(node) => self.root = delete_by(node, key, f),
            _ => self.root = None
        }
    }

    pub fn check(&self) -> bool {
        is_BST(&self.root, None, None) && is_AVL(&self.root)
    }

    pub fn cell_size() -> usize {
        heap::rc_size_for::<AVLNode<T, M>>()
    }
}

#[repr(C)]
struct AVLNode<T, M> where T : cmp::Ord, M : MemoryAllocator {
    value  : T,
    height : i64,
    left   : OptNodeBox<T, M>,
    right  : OptNodeBox<T, M>
}

impl<T, M> AVLNode<T, M> where T : cmp::Ord, M : MemoryAllocator {
    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn left(&self) -> OptNodeBox<T, M> {
        self.left.as_ref().map(|rc| heap::RC::clone(rc))
    }

    pub fn right(&self) -> OptNodeBox<T, M> {
        self.right.as_ref().map(|rc| heap::RC::clone(rc))
    }

    pub fn left_mut(&mut self) -> &mut OptNodeBox<T, M> {
        &mut self.left
    }

    pub fn right_mut(&mut self) -> &mut OptNodeBox<T, M> {
        &mut self.right
    }

    pub fn set_right(&mut self, v : OptNodeBox<T, M>) {
        self.right = v
    }

    pub fn set_left(&mut self, v : OptNodeBox<T, M>) {
        self.left = v
    }

    pub fn update_height(&mut self) {
        self.height = 1 + cmp::max(height(&self.left()), height(&self.right()))
    }

    pub fn take_left(&mut self) -> OptNodeBox<T, M> {
        self.left.take()
    }

    pub fn take_right(&mut self) -> OptNodeBox<T, M> {
        self.right.take()
    }

    pub fn take_left_unwrap(&mut self) -> NodeBox<T, M> {
        self.take_left().unwrap()
    }

    pub fn take_right_unwrap(&mut self) -> NodeBox<T, M> {
        self.take_right().unwrap()
    }

    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn new(value : T, height : i64, memory_allocator : &mut M) -> heap::RC<Self, M> where M : MemoryAllocator {
        let result = AVLNode {
            value  : value,
            height : height,
            left   : None,
            right  : None
        };

        heap::RC::new(result, memory_allocator)
    }
}