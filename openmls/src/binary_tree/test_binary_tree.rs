use std::collections::HashSet;

use crate::binary_tree::{MlsBinaryTree, MlsBinaryTreeDiffError, MlsBinaryTreeError};

use super::{array_representation::treemath::TreeNodeIndex, LeafNodeIndex};

#[test]
fn test_tree_basics() {
    // Test tree creation: Wrong number of nodes.
    let mut nodes = vec![1, 0];
    assert_eq!(
        MlsBinaryTree::new(nodes.clone())
            .expect_err("No error when creating a non-full binary tree."),
        MlsBinaryTreeError::InvalidNumberOfNodes
    );
    nodes.push(2);

    let tree1 = MlsBinaryTree::new(nodes.clone()).expect("Error when creating tree from nodes.");

    // Test size reporting
    assert_eq!(tree1.size(), 3);
    assert_eq!(tree1.leaf_count(), 2);

    // Test tree creation: Too many nodes (only in cases where usize is 64 bit).
    #[cfg(target_pointer_width = "64")]
    // We allow uninitialized vectors because we don't want to allocate so much memory
    #[allow(clippy::uninit_vec)]
    unsafe {
        let len = u32::MAX as usize + 2;
        let mut nodes: Vec<u32> = Vec::new();

        nodes.set_len(len);

        assert_eq!(
            MlsBinaryTree::new(nodes).expect_err("No error while creating too large tree."),
            MlsBinaryTreeError::OutOfRange
        )
    }

    // Test node export
    let exported_nodes = tree1.nodes().map(|(_, node)| *node).collect();
    let tree2 =
        MlsBinaryTree::new(exported_nodes).expect("error when creating tree from exported nodes.");

    assert_eq!(tree1, tree2);

    // Node access
    assert_eq!(&1, tree1.node_by_index(TreeNodeIndex::new(0)));
    assert_eq!(&0, tree1.node_by_index(TreeNodeIndex::new(1)));
    assert_eq!(&2, tree1.node_by_index(TreeNodeIndex::new(2)));

    // Leaves
    let leaves1: Vec<(LeafNodeIndex, &u32)> = tree1.leaves().collect();
    assert_eq!(
        vec![(LeafNodeIndex::new(0), &1), (LeafNodeIndex::new(1), &2)],
        leaves1
    );

    let tree3 = MlsBinaryTree::new(vec![1]).expect("error creating 1 node binary tree.");
    let leaves3: Vec<(LeafNodeIndex, &u32)> = tree3.leaves().collect();
    assert_eq!(vec![(LeafNodeIndex::new(0), &1)], leaves3);
}

#[test]
fn test_diff_merging() {
    let mut tree = MlsBinaryTree::new(vec![2, 0, 4]).expect("Error creating tree.");
    let original_tree = tree.clone();

    // Test the leaves in the original tree
    let leaves: Vec<(LeafNodeIndex, &u32)> = original_tree.leaves().collect();

    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0], (LeafNodeIndex::new(0), &2));
    assert_eq!(leaves[1], (LeafNodeIndex::new(1), &4));

    let mut diff = tree.empty_diff();

    // Merging larger diffs.

    // Add a lot of leaves.
    for index in 0..1000 {
        diff.add_leaf(index, index)
            .expect("error while adding large number of leaves");
    }

    // Check that the leaves were actually added.
    let leaves: Vec<(LeafNodeIndex, &u32)> = diff.leaves().collect();

    // Expect original 2 leaves + 1000 new ones
    assert_eq!(leaves.len(), 2 + 1000);

    // Expect original leaves
    assert_eq!(leaves[0], (LeafNodeIndex::new(0), &2));
    assert_eq!(leaves[1], (LeafNodeIndex::new(1), &4));

    // Expect new leaves
    assert_eq!(leaves[2], (LeafNodeIndex::new(2), &0));
    assert_eq!(leaves[3], (LeafNodeIndex::new(3), &1));
    assert_eq!(leaves[4], (LeafNodeIndex::new(4), &2));

    let first_leaf = leaves.first().expect("leaf vector is empty");
    let last_leaf = leaves.last().expect("leaf vector is empty");
    assert_eq!(first_leaf, &(LeafNodeIndex::new(0), &2));
    assert_eq!(last_leaf, &(LeafNodeIndex::new(1001), &999));
    assert_eq!(leaves.len(), diff.leaf_count() as usize);

    // Remove some of them again
    for _ in 0..200 {
        diff.remove_leaf()
            .expect("error while removing large number of leaves");
    }

    // Check that the leaves were actually removed.
    let leaves: Vec<(LeafNodeIndex, &u32)> = diff.leaves().collect();

    let first_leaf = leaves.first().expect("leaf vector is empty");
    let last_leaf = leaves.last().expect("leaf vector is empty");
    assert_eq!(first_leaf, &(LeafNodeIndex::new(0), &2));
    assert_eq!(last_leaf, &(LeafNodeIndex::new(801), &799));
    assert_eq!(leaves.len(), diff.leaf_count() as usize);

    let staged_diff = diff.into();
    tree.merge_diff(staged_diff);

    // Verify that the tree has changed post-merge.
    let leaves: Vec<(LeafNodeIndex, &u32)> = tree.leaves().collect();

    let first_leaf = leaves.first().expect("leaf vector is empty");
    let last_leaf = leaves.last().expect("leaf vector is empty");
    assert_eq!(first_leaf, &(LeafNodeIndex::new(0), &2));
    assert_eq!(last_leaf, &(LeafNodeIndex::new(801), &799));

    // Merging a diff that decreases the size of the tree.

    let mut diff = tree.empty_diff();
    for _ in 0..800 {
        diff.remove_leaf()
            .expect("error while removing large number of leaves");
    }

    let staged_diff = diff.into();
    tree.merge_diff(staged_diff);

    assert_eq!(tree, original_tree);
}

#[test]
fn test_leaf_addition_and_removal_errors() {
    let tree = MlsBinaryTree::new((0..3).collect()).expect("error creating tree");
    let mut diff = tree.empty_diff();

    diff.remove_leaf().expect("error removing leaf");

    // Should fail removing the last remaining leaf.
    assert_eq!(
        diff.remove_leaf()
            .expect_err("no error trying to remove the last leaf in the diff"),
        MlsBinaryTreeDiffError::TreeTooSmall
    );

    // Let's test what happens when the tree is getting too large.
    let mut nodes: Vec<u32> = Vec::new();

    // We allow uninitialized vectors because we don't want to allocate so much memory
    #[allow(clippy::uninit_vec)]
    unsafe {
        nodes.set_len(u32::MAX as usize);

        let tree = MlsBinaryTree::new(nodes).expect("error creating tree");
        let mut diff = tree.empty_diff();

        assert_eq!(
            diff.add_leaf(666, 667)
                .expect_err("no error adding beyond u32 max"),
            MlsBinaryTreeDiffError::TreeTooLarge
        )
    }
}

#[test]
fn test_diff_iter() {
    let tree = MlsBinaryTree::new((0..101).collect()).expect("error creating tree");

    let diff = tree.empty_diff();

    let mut node_set = HashSet::new();

    for (_, node) in diff.nodes() {
        node_set.insert(node);
    }

    for i in 0..101 {
        assert!(node_set.contains(&i));
    }
}

#[test]
fn test_export_diff_nodes() {
    let tree = MlsBinaryTree::new((0..101).collect()).expect("error creating tree");

    let diff = tree.empty_diff();

    let nodes = diff.nodes().map(|(_, node)| *node).collect();

    // If we re-export the nodes into a tree, we should end up with the same tree.
    let new_tree = MlsBinaryTree::new(nodes).expect("error creating tree from exported nodes");

    assert_eq!(tree, new_tree);
}

#[test]
fn test_diff_mutable_access_after_manipulation() {
    let tree = MlsBinaryTree::new((0..101).collect()).expect("error creating tree");

    let mut diff = tree.empty_diff();

    // Let's change the nodes along a direct path.
    diff.set_direct_path_to_node(LeafNodeIndex::new(5), &999);

    // Now let's get references to a neighbour's path, where some nodes were
    // changed and some weren't.
    let direct_path_refs = diff
        .direct_path(LeafNodeIndex::new(6))
        .iter()
        .map(|index| TreeNodeIndex::Parent(*index))
        .collect::<Vec<TreeNodeIndex>>();
    for node_ref in &direct_path_refs {
        let node_mut = diff.node_mut(*node_ref);
        *node_mut = 888;
    }

    let direct_path = diff
        .deref_vec(direct_path_refs)
        .expect("error dereferencing direct path nodes");
    assert_eq!(direct_path, vec![&888, &888, &888, &888, &888, &888])
}
