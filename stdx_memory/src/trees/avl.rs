use core::marker;
use core::mem;
use core::ops;
use core::iter;
use core::ptr;
use core::cmp;
use core::cmp::Ordering;
use MemoryAllocator;
use heap;
use smart_ptr;
use stdx::Sequence;
use stdx::Iterable;
use core::ops::Deref;
use core::ops::DerefMut;

type OptNodeBox<T> = Option<heap::Box<AVLNode<T>>>;
type NodeBox<T> = heap::Box<AVLNode<T>>;


fn height<T>(node : &OptNodeBox<T>) -> i64 where T : cmp::Ord {
    node.as_ref()
        .map(|n| n.height())
        .unwrap_or(-1)
}

fn insert0<T, M>(mut node : NodeBox<T>, value : T, memory_allocator : &mut M) -> NodeBox<T> where T : cmp::Ord + Copy, M : MemoryAllocator  {
    let cmp_result = node.value().cmp(&value);
            
    match cmp_result {
        Ordering::Less    => {
            if node.left.is_some() {
                let new_left = insert0(node.take_left_unwrap(), value, memory_allocator);
                node.set_left(Some(new_left));
                
            }
            else {
                node.set_left(Some(AVLNode::new(value, 0, memory_allocator)));
            }
        },
        Ordering::Greater => {
            if node.right.is_some() {
                let new_right = insert0(node.take_right_unwrap(), value, memory_allocator);
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


fn find0<T>(node : &OptNodeBox<T>, x : T) -> Option<T> where T : cmp::Ord + Copy {
    node.as_ref().and_then(|n| match n.value().cmp(&x) {
            Ordering::Less    => find0(&n.left, x),
            Ordering::Greater => find0(&n.right, x),
            _                 => Some(n.value)
    })
}

fn rotate_right<T>(mut x : NodeBox<T>) -> NodeBox<T> where T : cmp::Ord {
    let mut y = x.take_left().expect("invalid avl");

    x.set_left(y.take_right());
    x.update_height();
        
    y.set_right(Some(heap::Box::from_pointer(&x)));
    y.update_height();

    y
}

fn rotate_left<T>(mut x : NodeBox<T>) -> NodeBox<T> where T : cmp::Ord {
    let mut y = x.take_right().expect("invalid avl");

    x.set_right(y.take_left());
    y.set_left(Some(heap::Box::from_pointer(&x)));

    x.update_height();
    y.update_height();

    y
}

fn balance_factor_opt<T>(node : &OptNodeBox<T>) -> i64 where T : cmp::Ord {
    node.as_ref()
        .map(|n| balance_factor(n))
        .unwrap_or(0)
}

fn balance_factor<T>(node : &AVLNode<T>) -> i64 where T : cmp::Ord {
    height(&node.left) - height(&node.right)
}

fn balance<T>(mut node : NodeBox<T>) -> NodeBox<T> where T : cmp::Ord {
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

fn min<T>(node : &NodeBox<T>) -> &NodeBox<T> where T : cmp::Ord {
    node.left()
        .as_ref()
        .map(|l| min(l))
        .unwrap_or(node)
}

fn delete_min<T>(mut node : NodeBox<T>) -> NodeBox<T> where T : cmp::Ord {
    match node.take_left() {
        Some(left) => {
            node.set_left(Some(delete_min(left)));
            node.update_height();

            balance(node)
        },
        _ => node.take_right().expect("Invalid avl (no right node) in delete min"),
    }
}

fn delete<T>(mut node : NodeBox<T>, value : T) -> NodeBox<T> where T : cmp::Ord + Copy {
    let cmp_result = node.value().cmp(&value);
            
    match cmp_result {
        Ordering::Less if node.left().is_some() => {
            let new_left = delete(node.take_left_unwrap(), value);
            node.set_left(Some(new_left));
        },
        Ordering::Greater if node.right().is_some() => {
            let new_right = delete(node.take_right_unwrap(), value);
            node.set_right(Some(new_right));                
        },
        _ => {
            if node.left().is_none() && node.right().is_some() {
                return node.take_right_unwrap()
            }
            else if node.right().is_none() && node.left().is_some() {
                return node.take_left_unwrap()
            }
            else {
                let node_copy = heap::Box::from_pointer(node.deref());

                {
                    let new_node = min(node_copy.right().as_ref().unwrap());

                    node = heap::Box::from_pointer(new_node);
                }

                let right_copy = heap::Box::from_pointer(node_copy.right().as_ref().unwrap().deref());
                //let left_copy = heap::Box::from_pointer(node_copy.left().as_ref().unwrap().deref());
                let new_right = delete_min(right_copy);                
                node.set_right(Some(new_right));
                //node.set_left(Some(left_copy));


                /*
                Node y = x;
                x = min(y.right);
                x.right = deleteMin(y.right);
                x.left = y.left;
                */
            }
        }
    }
        
    node.update_height();
    balance(node)
}

fn is_BST<T>(node : &OptNodeBox<T>, min : Option<T>, max : Option<T>) -> bool where T : cmp::Ord + Copy {
    node.as_ref()
        .map(|n| {
            let value = n.value();
            let min_check = min.is_none() || value <= min.unwrap();
            let max_check = max.is_none() || value >= max.unwrap();
            
            
            min_check && 
            max_check && 
            is_BST(n.left(), min, Some(value)) && 
            is_BST(n.right(), Some(value), max)
        })
        .unwrap_or(true)
}

fn is_AVL<T>(node : &OptNodeBox<T>) -> bool where T : cmp::Ord + Copy {
    node.as_ref()
        .map(|n| {
            let balance_factor = balance_factor(n);
            let balance_factor_check = balance_factor >= -1 && balance_factor <= 1;

            balance_factor_check &&
            is_AVL(n.left()) &&
            is_AVL(n.right())
        })
        .unwrap_or(true)
}

pub struct AVLTree<T> where T : cmp::Ord {
    root : OptNodeBox<T>
}

impl<T> AVLTree<T> where T : cmp::Ord + Copy {
    pub fn find(&self, x : T) -> Option<T> {
        find0(&self.root, x)
    }

    pub fn height(&self) -> i64 {
        height(&self.root)
    }

    pub fn insert<M>(&mut self, value : T, memory_allocator : &mut M) where M : MemoryAllocator {
        match self.root.take() {
            Some(node) => self.root = Some(insert0(node, value, memory_allocator)),
            _ => self.root = Some(AVLNode::new(value, 0, memory_allocator)) 
        }
    }

    pub fn delete(&mut self, key : T) {
        match self.root.take() {
            Some(node) => self.root = Some(delete(node, key)),
            _ => self.root = None
        }
    }

    pub fn check(&self) -> bool {
        is_BST(&self.root, None, None) && is_AVL(&self.root)
    }
}

impl<T> AVLTree<T> where T : cmp::Ord {
    pub fn cell_size() -> usize {
        mem::size_of::<AVLNode<T>>()
    }
}

struct AVLNode<T> where T : cmp::Ord {
    value  : T,
    height : i64,
    left   : OptNodeBox<T>,
    right  : OptNodeBox<T>
}

impl<T> AVLNode<T> where T : cmp::Ord {
    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn left(&self) -> &OptNodeBox<T> {
        &self.left
    }

    pub fn right(&self) -> &OptNodeBox<T> {
        &self.right
    }

    pub fn set_right(&mut self, v : OptNodeBox<T>) {
        self.right = v
    }

    pub fn set_left(&mut self, v : OptNodeBox<T>) {
        self.left = v
    }

    pub fn update_height(&mut self) {
        self.height = 1 + cmp::max(height(self.left()), height(self.right()))
    }

    pub fn take_left(&mut self) -> OptNodeBox<T> {
        self.left.take()
    }

    pub fn take_right(&mut self) -> OptNodeBox<T> {
        self.right.take()
    }

    pub fn take_left_unwrap(&mut self) -> NodeBox<T> {
        self.take_left().unwrap()
    }

    pub fn take_right_unwrap(&mut self) -> NodeBox<T> {
        self.take_right().unwrap()
    }

    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }
}

impl<T> AVLNode<T> where T : cmp::Ord + Copy {
    
    pub fn value(&self) -> T {
        self.value
    }

    pub fn new<M>(value : T, height : i64, memory_allocator : &mut M) -> heap::Box<Self> where M : MemoryAllocator {
        let result = AVLNode {
            value  : value,
            height : height,
            left   : None,
            right  : None
        };

        heap::Box::new(result, memory_allocator)
    }
}
/*
public class AVLTreeST<Key extends Comparable<Key>, Value> {

    /**
     * The root node.
     */
    private Node root;

    /**
     * This class represents an inner node of the AVL tree.
     */
    private class Node {
        private final Key key;   // the key
        private Value val;       // the associated value
        private int height;      // height of the subtree
        private int size;        // number of nodes in subtree
        private Node left;       // left subtree
        private Node right;      // right subtree

        public Node(Key key, Value val, int height, int size) {
            this.key = key;
            this.val = val;
            this.size = size;
            this.height = height;
        }
    }

    /**
     * Initializes an empty symbol table.
     */
    public AVLTreeST() {
    }

    /**
     * Checks if the symbol table is empty.
     * 
     * @return {@code true} if the symbol table is empty.
     */
    public boolean isEmpty() {
        return root == null;
    }

    /**
     * Returns the number key-value pairs in the symbol table.
     * 
     * @return the number key-value pairs in the symbol table
     */
    public int size() {
        return size(root);
    }

    /**
     * Returns the number of nodes in the subtree.
     * 
     * @param x the subtree
     * 
     * @return the number of nodes in the subtree
     */
    private int size(Node x) {
        if (x == null) return 0;
        return x.size;
    }

    /**
     * Inserts the specified key-value pair into the symbol table, overwriting
     * the old value with the new value if the symbol table already contains the
     * specified key. Deletes the specified key (and its associated value) from
     * this symbol table if the specified value is {@code null}.
     * 
     * @param key the key
     * @param val the value
     * @throws IllegalArgumentException if {@code key} is {@code null}
     */
    public void put(Key key, Value val) {
        if (key == null) throw new IllegalArgumentException("first argument to put() is null");
        if (val == null) {
            delete(key);
            return;
        }
        root = put(root, key, val);
        assert check();
    }

    /**
     * Inserts the key-value pair in the subtree. It overrides the old value
     * with the new value if the symbol table already contains the specified key
     * and deletes the specified key (and its associated value) from this symbol
     * table if the specified value is {@code null}.
     * 
     * @param x the subtree
     * @param key the key
     * @param val the value
     * @return the subtree
     */
    private Node put(Node x, Key key, Value val) {
        if (x == null) return new Node(key, val, 0, 1);
        int cmp = key.compareTo(x.key);
        if (cmp < 0) {
            x.left = put(x.left, key, val);
        }
        else if (cmp > 0) {
            x.right = put(x.right, key, val);
        }
        else {
            x.val = val;
            return x;
        }
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        return balance(x);
    }

    /**
     * Restores the AVL tree property of the subtree.
     * 
     * @param x the subtree
     * @return the subtree with restored AVL property
     */
    private Node balance(Node x) {
        if (balanceFactor(x) < -1) {
            if (balanceFactor(x.right) > 0) {
                x.right = rotateRight(x.right);
            }
            x = rotateLeft(x);
        }
        else if (balanceFactor(x) > 1) {
            if (balanceFactor(x.left) < 0) {
                x.left = rotateLeft(x.left);
            }
            x = rotateRight(x);
        }
        return x;
    }

    /**
     * Returns the balance factor of the subtree. The balance factor is defined
     * as the difference in height of the left subtree and right subtree, in
     * this order. Therefore, a subtree with a balance factor of -1, 0 or 1 has
     * the AVL property since the heights of the two child subtrees differ by at
     * most one.
     * 
     * @param x the subtree
     * @return the balance factor of the subtree
     */
    private int balanceFactor(Node x) {
        return height(x.left) - height(x.right);
    }

    /**
     * Rotates the given subtree to the right.
     * 
     * @param x the subtree
     * @return the right rotated subtree
     */
    private Node rotateRight(Node x) {
        Node y = x.left;
        x.left = y.right;
        y.right = x;
        y.size = x.size;
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        y.height = 1 + Math.max(height(y.left), height(y.right));
        return y;
    }

    /**
     * Rotates the given subtree to the left.
     * 
     * @param x the subtree
     * @return the left rotated subtree
     */
    private Node rotateLeft(Node x) {
        Node y = x.right;
        x.right = y.left;
        y.left = x;
        y.size = x.size;
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        y.height = 1 + Math.max(height(y.left), height(y.right));
        return y;
    }

    /**
     * Removes the specified key and its associated value from the symbol table
     * (if the key is in the symbol table).
     * 
     * @param key the key
     * @throws IllegalArgumentException if {@code key} is {@code null}
     */
    public void delete(Key key) {
        if (key == null) throw new IllegalArgumentException("argument to delete() is null");
        if (!contains(key)) return;
        root = delete(root, key);
        assert check();
    }

    /**
     * Removes the specified key and its associated value from the given
     * subtree.
     * 
     * @param x the subtree
     * @param key the key
     * @return the updated subtree
     */
    private Node delete(Node x, Key key) {
        int cmp = key.compareTo(x.key);
        if (cmp < 0) {
            x.left = delete(x.left, key);
        }
        else if (cmp > 0) {
            x.right = delete(x.right, key);
        }
        else {
            if (x.left == null) {
                return x.right;
            }
            else if (x.right == null) {
                return x.left;
            }
            else {
                Node y = x;
                x = min(y.right);
                x.right = deleteMin(y.right);
                x.left = y.left;
            }
        }
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        return balance(x);
    }

    /**
     * Removes the smallest key and associated value from the symbol table.
     * 
     * @throws NoSuchElementException if the symbol table is empty
     */
    public void deleteMin() {
        if (isEmpty()) throw new NoSuchElementException("called deleteMin() with empty symbol table");
        root = deleteMin(root);
        assert check();
    }

    /**
     * Removes the smallest key and associated value from the given subtree.
     * 
     * @param x the subtree
     * @return the updated subtree
     */
    private Node deleteMin(Node x) {
        if (x.left == null) return x.right;
        x.left = deleteMin(x.left);
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        return balance(x);
    }

    /**
     * Removes the largest key and associated value from the symbol table.
     * 
     * @throws NoSuchElementException if the symbol table is empty
     */
    public void deleteMax() {
        if (isEmpty()) throw new NoSuchElementException("called deleteMax() with empty symbol table");
        root = deleteMax(root);
        assert check();
    }

    /**
     * Removes the largest key and associated value from the given subtree.
     * 
     * @param x the subtree
     * @return the updated subtree
     */
    private Node deleteMax(Node x) {
        if (x.right == null) return x.left;
        x.right = deleteMax(x.right);
        x.size = 1 + size(x.left) + size(x.right);
        x.height = 1 + Math.max(height(x.left), height(x.right));
        return balance(x);
    }

    /**
     * Returns the smallest key in the symbol table.
     * 
     * @return the smallest key in the symbol table
     * @throws NoSuchElementException if the symbol table is empty
     */
    public Key min() {
        if (isEmpty()) throw new NoSuchElementException("called min() with empty symbol table");
        return min(root).key;
    }

    /**
     * Returns the node with the smallest key in the subtree.
     * 
     * @param x the subtree
     * @return the node with the smallest key in the subtree
     */
    private Node min(Node x) {
        if (x.left == null) return x;
        return min(x.left);
    }

    /**
     * Returns the largest key in the symbol table.
     * 
     * @return the largest key in the symbol table
     * @throws NoSuchElementException if the symbol table is empty
     */
    public Key max() {
        if (isEmpty()) throw new NoSuchElementException("called max() with empty symbol table");
        return max(root).key;
    }

    /**
     * Returns the node with the largest key in the subtree.
     * 
     * @param x the subtree
     * @return the node with the largest key in the subtree
     */
    private Node max(Node x) {
        if (x.right == null) return x;
        return max(x.right);
    }

    /**
     * Returns the largest key in the symbol table less than or equal to
     * {@code key}.
     * 
     * @param key the key
     * @return the largest key in the symbol table less than or equal to
     *         {@code key}
     * @throws NoSuchElementException if the symbol table is empty
     * @throws IllegalArgumentException if {@code key} is {@code null}
     */
    public Key floor(Key key) {
        if (key == null) throw new IllegalArgumentException("argument to floor() is null");
        if (isEmpty()) throw new NoSuchElementException("called floor() with empty symbol table");
        Node x = floor(root, key);
        if (x == null) return null;
        else return x.key;
    }

    /**
     * Returns the node in the subtree with the largest key less than or equal
     * to the given key.
     * 
     * @param x the subtree
     * @param key the key
     * @return the node in the subtree with the largest key less than or equal
     *         to the given key
     */
    private Node floor(Node x, Key key) {
        if (x == null) return null;
        int cmp = key.compareTo(x.key);
        if (cmp == 0) return x;
        if (cmp < 0) return floor(x.left, key);
        Node y = floor(x.right, key);
        if (y != null) return y;
        else return x;
    }

    /**
     * Returns the smallest key in the symbol table greater than or equal to
     * {@code key}.
     * 
     * @param key the key
     * @return the smallest key in the symbol table greater than or equal to
     *         {@code key}
     * @throws NoSuchElementException if the symbol table is empty
     * @throws IllegalArgumentException if {@code key} is {@code null}
     */
    public Key ceiling(Key key) {
        if (key == null) throw new IllegalArgumentException("argument to ceiling() is null");
        if (isEmpty()) throw new NoSuchElementException("called ceiling() with empty symbol table");
        Node x = ceiling(root, key);
        if (x == null) return null;
        else return x.key;
    }

    /**
     * Returns the node in the subtree with the smallest key greater than or
     * equal to the given key.
     * 
     * @param x the subtree
     * @param key the key
     * @return the node in the subtree with the smallest key greater than or
     *         equal to the given key
     */
    private Node ceiling(Node x, Key key) {
        if (x == null) return null;
        int cmp = key.compareTo(x.key);
        if (cmp == 0) return x;
        if (cmp > 0) return ceiling(x.right, key);
        Node y = ceiling(x.left, key);
        if (y != null) return y;
        else return x;
    }

    /**
     * Returns the kth smallest key in the symbol table.
     * 
     * @param k the order statistic
     * @return the kth smallest key in the symbol table
     * @throws IllegalArgumentException unless {@code k} is between 0 and
     *             {@code size() -1 }
     */
    public Key select(int k) {
        if (k < 0 || k >= size()) throw new IllegalArgumentException("k is not in range 0-" + (size() - 1));
        Node x = select(root, k);
        return x.key;
    }

    /**
     * Returns the node with key the kth smallest key in the subtree.
     * 
     * @param x the subtree
     * @param k the kth smallest key in the subtree
     * @return the node with key the kth smallest key in the subtree
     */
    private Node select(Node x, int k) {
        if (x == null) return null;
        int t = size(x.left);
        if (t > k) return select(x.left, k);
        else if (t < k) return select(x.right, k - t - 1);
        else return x;
    }

    /**
     * Returns the number of keys in the symbol table strictly less than
     * {@code key}.
     * 
     * @param key the key
     * @return the number of keys in the symbol table strictly less than
     *         {@code key}
     * @throws IllegalArgumentException if {@code key} is {@code null}
     */
    public int rank(Key key) {
        if (key == null) throw new IllegalArgumentException("argument to rank() is null");
        return rank(key, root);
    }

    /**
     * Returns the number of keys in the subtree less than key.
     * 
     * @param key the key
     * @param x the subtree
     * @return the number of keys in the subtree less than key
     */
    private int rank(Key key, Node x) {
        if (x == null) return 0;
        int cmp = key.compareTo(x.key);
        if (cmp < 0) return rank(key, x.left);
        else if (cmp > 0) return 1 + size(x.left) + rank(key, x.right);
        else return size(x.left);
    }

    /**
     * Returns all keys in the symbol table.
     * 
     * @return all keys in the symbol table
     */
    public Iterable<Key> keys() {
        return keysInOrder();
    }

    /**
     * Returns all keys in the symbol table following an in-order traversal.
     * 
     * @return all keys in the symbol table following an in-order traversal
     */
    public Iterable<Key> keysInOrder() {
        Queue<Key> queue = new Queue<Key>();
        keysInOrder(root, queue);
        return queue;
    }

    /**
     * Adds the keys in the subtree to queue following an in-order traversal.
     * 
     * @param x the subtree
     * @param queue the queue
     */
    private void keysInOrder(Node x, Queue<Key> queue) {
        if (x == null) return;
        keysInOrder(x.left, queue);
        queue.enqueue(x.key);
        keysInOrder(x.right, queue);
    }

    /**
     * Returns all keys in the symbol table following a level-order traversal.
     * 
     * @return all keys in the symbol table following a level-order traversal.
     */
    public Iterable<Key> keysLevelOrder() {
        Queue<Key> queue = new Queue<Key>();
        if (!isEmpty()) {
            Queue<Node> queue2 = new Queue<Node>();
            queue2.enqueue(root);
            while (!queue2.isEmpty()) {
                Node x = queue2.dequeue();
                queue.enqueue(x.key);
                if (x.left != null) {
                    queue2.enqueue(x.left);
                }
                if (x.right != null) {
                    queue2.enqueue(x.right);
                }
            }
        }
        return queue;
    }

    /**
     * Returns all keys in the symbol table in the given range.
     * 
     * @param lo the lowest key
     * @param hi the highest key
     * @return all keys in the symbol table between {@code lo} (inclusive)
     *         and {@code hi} (exclusive)
     * @throws IllegalArgumentException if either {@code lo} or {@code hi}
     *             is {@code null}
     */
    public Iterable<Key> keys(Key lo, Key hi) {
        if (lo == null) throw new IllegalArgumentException("first argument to keys() is null");
        if (hi == null) throw new IllegalArgumentException("second argument to keys() is null");
        Queue<Key> queue = new Queue<Key>();
        keys(root, queue, lo, hi);
        return queue;
    }

    /**
     * Adds the keys between {@code lo} and {@code hi} in the subtree
     * to the {@code queue}.
     * 
     * @param x the subtree
     * @param queue the queue
     * @param lo the lowest key
     * @param hi the highest key
     */
    private void keys(Node x, Queue<Key> queue, Key lo, Key hi) {
        if (x == null) return;
        int cmplo = lo.compareTo(x.key);
        int cmphi = hi.compareTo(x.key);
        if (cmplo < 0) keys(x.left, queue, lo, hi);
        if (cmplo <= 0 && cmphi >= 0) queue.enqueue(x.key);
        if (cmphi > 0) keys(x.right, queue, lo, hi);
    }

    /**
     * Returns the number of keys in the symbol table in the given range.
     * 
     * @param lo minimum endpoint
     * @param hi maximum endpoint
     * @return the number of keys in the symbol table between {@code lo}
     *         (inclusive) and {@code hi} (exclusive)
     * @throws IllegalArgumentException if either {@code lo} or {@code hi}
     *             is {@code null}
     */
    public int size(Key lo, Key hi) {
        if (lo == null) throw new IllegalArgumentException("first argument to size() is null");
        if (hi == null) throw new IllegalArgumentException("second argument to size() is null");
        if (lo.compareTo(hi) > 0) return 0;
        if (contains(hi)) return rank(hi) - rank(lo) + 1;
        else return rank(hi) - rank(lo);
    }

    /**
     * Checks if the AVL tree invariants are fine.
     * 
     * @return {@code true} if the AVL tree invariants are fine
     */
    private boolean check() {
        if (!isBST()) StdOut.println("Symmetric order not consistent");
        if (!isAVL()) StdOut.println("AVL property not consistent");
        if (!isSizeConsistent()) StdOut.println("Subtree counts not consistent");
        if (!isRankConsistent()) StdOut.println("Ranks not consistent");
        return isBST() && isAVL() && isSizeConsistent() && isRankConsistent();
    }

    /**
     * Checks if AVL property is consistent.
     * 
     * @return {@code true} if AVL property is consistent.
     */
    private boolean isAVL() {
        return isAVL(root);
    }

    /**
     * Checks if AVL property is consistent in the subtree.
     * 
     * @param x the subtree
     * @return {@code true} if AVL property is consistent in the subtree
     */
    private boolean isAVL(Node x) {
        if (x == null) return true;
        int bf = balanceFactor(x);
        if (bf > 1 || bf < -1) return false;
        return isAVL(x.left) && isAVL(x.right);
    }

    /**
     * Checks if the symmetric order is consistent.
     * 
     * @return {@code true} if the symmetric order is consistent
     */
    private boolean isBST() {
        return isBST(root, null, null);
    }

    /**
     * Checks if the tree rooted at x is a BST with all keys strictly between
     * min and max (if min or max is null, treat as empty constraint) Credit:
     * Bob Dondero's elegant solution
     * 
     * @param x the subtree
     * @param min the minimum key in subtree
     * @param max the maximum key in subtree
     * @return {@code true} if if the symmetric order is consistent
     */
    private boolean isBST(Node x, Key min, Key max) {
        if (x == null) return true;
        if (min != null && x.key.compareTo(min) <= 0) return false;
        if (max != null && x.key.compareTo(max) >= 0) return false;
        return isBST(x.left, min, x.key) && isBST(x.right, x.key, max);
    }

    /**
     * Checks if size is consistent.
     * 
     * @return {@code true} if size is consistent
     */
    private boolean isSizeConsistent() {
        return isSizeConsistent(root);
    }

    /**
     * Checks if the size of the subtree is consistent.
     * 
     * @return {@code true} if the size of the subtree is consistent
     */
    private boolean isSizeConsistent(Node x) {
        if (x == null) return true;
        if (x.size != size(x.left) + size(x.right) + 1) return false;
        return isSizeConsistent(x.left) && isSizeConsistent(x.right);
    }

    /**
     * Checks if rank is consistent.
     * 
     * @return {@code true} if rank is consistent
     */
    private boolean isRankConsistent() {
        for (int i = 0; i < size(); i++)
            if (i != rank(select(i))) return false;
        for (Key key : keys())
            if (key.compareTo(select(rank(key))) != 0) return false;
        return true;
    }

    /**
     * Unit tests the {@code AVLTreeST} data type.
     *
     * @param args the command-line arguments
     */
    public static void main(String[] args) {
        AVLTreeST<String, Integer> st = new AVLTreeST<String, Integer>();
        for (int i = 0; !StdIn.isEmpty(); i++) {
            String key = StdIn.readString();
            st.put(key, i);
        }
        for (String s : st.keys())
            StdOut.println(s + " " + st.get(s));
        StdOut.println();
    }
}
*/