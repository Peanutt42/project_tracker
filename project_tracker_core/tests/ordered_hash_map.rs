use project_tracker_core::OrderedHashMap;

#[test]
fn test_order_after_moving() {
	let mut hash_map = OrderedHashMap::new();
	hash_map.insert(1, 1);
	hash_map.insert(2, 2);
	hash_map.insert(3, 3);
	for (i, (key, value)) in hash_map.iter().enumerate() {
		assert_eq!(key, *value);
		assert_eq!(i + 1, key);
	}
	hash_map.move_to(2, 0);
	assert_eq!(hash_map.get_at_order(0), hash_map.get(&2));
	hash_map.move_to(2, 2);
	assert_eq!(hash_map.get_at_order(2), hash_map.get(&2));
	hash_map.move_up(&2);
	hash_map.move_up(&2);
	assert_eq!(hash_map.get_at_order(0), hash_map.get(&2));
	hash_map.move_down(&2);
	hash_map.move_down(&2);
	assert_eq!(hash_map.get_at_order(2), hash_map.get(&2));

	// should result in the original hash_map
	hash_map.move_up(&2);
	for (i, (key, value)) in hash_map.iter().enumerate() {
		assert_eq!(key, *value);
		assert_eq!(i + 1, key);
	}
}

#[test]
fn test_order_after_insertion_and_deletion() {
	let mut hash_map = OrderedHashMap::new();
	hash_map.insert(1, 1);
	hash_map.insert(2, 2);
	hash_map.insert(3, 3);

	hash_map.insert(4, 4);
	hash_map.move_to(4, 1);
	hash_map.remove(&4);

	assert_eq!(hash_map.len(), 3);
	for (i, (key, _value)) in hash_map.iter().enumerate() {
		assert_eq!(i + 1, key);
	}
}

#[test]
fn test_double_reversing_order() {
	let n = 1000;
	let mut hash_map = OrderedHashMap::new();
	for i in 0..n {
		hash_map.insert(i, i);
	}
	let original = hash_map.clone();
	// double reversing
	for i in 0..n {
		hash_map.move_to(i, n - hash_map.get_order(&i).unwrap() - 1);
	}
	for i in 0..n {
		hash_map.move_to(i, n - hash_map.get_order(&i).unwrap() - 1);
	}
	assert_eq!(original, hash_map);
}

#[test]
fn test_move_before_other_first_order() {
	let mut hash_map = OrderedHashMap::new();
	hash_map.insert(1, 5);
	hash_map.insert(2, 15);
	hash_map.move_before_other(2, 1);
	assert_eq!(
		hash_map.get_order(&2).unwrap(),
		hash_map.get_order(&1).unwrap() - 1
	);
}

#[test]
fn test_move_before_other_second_order() {
	let mut hash_map = OrderedHashMap::new();
	hash_map.insert(1, 5);
	hash_map.insert(2, 15);
	hash_map.insert(3, 25);
	hash_map.move_before_other(2, 3);
	assert_eq!(
		hash_map.get_order(&2).unwrap(),
		hash_map.get_order(&3).unwrap() - 1
	);
}

#[test]
fn test_move_to_end() {
	let mut hash_map = OrderedHashMap::new();

	hash_map.insert(1, 5);
	hash_map.insert(2, 15);
	hash_map.insert(3, 25);
	hash_map.insert(4, 35);

	hash_map.move_to_end(&2);

	let mut iter = hash_map.iter();
	assert_eq!(iter.next().unwrap().0, 1);
	assert_eq!(iter.next().unwrap().0, 3);
	assert_eq!(iter.next().unwrap().0, 4);
	assert_eq!(iter.next().unwrap().0, 2);
}

#[test]
fn test_append_ordered_hash_maps() {
	let mut ordered_hash_map = OrderedHashMap::new();
	ordered_hash_map.insert(1, 2);
	ordered_hash_map.insert(3, 4);
	ordered_hash_map.insert(5, 6);

	let mut other_ordered_hash_map = OrderedHashMap::new();
	other_ordered_hash_map.insert(7, 8);
	other_ordered_hash_map.insert(9, 10);
	other_ordered_hash_map.insert(11, 12);

	let mut expected_hash_map = OrderedHashMap::new();
	expected_hash_map.insert(1, 2);
	expected_hash_map.insert(3, 4);
	expected_hash_map.insert(5, 6);
	expected_hash_map.insert(7, 8);
	expected_hash_map.insert(9, 10);
	expected_hash_map.insert(11, 12);

	ordered_hash_map.append(&mut other_ordered_hash_map);

	assert!(other_ordered_hash_map.is_empty());
	assert_eq!(expected_hash_map, ordered_hash_map);
}
