mod nohash;

use std::collections::HashMap;

pub type NoHashMap<K, V> = HashMap<K, V, nohash::BuildNoHashDefault<K>>;

#[cfg(test)]
mod tests {
    use super::*;
    use insim::identifiers::{ConnectionId, PlayerId};

    #[test]
    fn test_nohashmap_bool() {
        let mut map = NoHashMap::default();
        map.insert(PlayerId(1), true);

        assert_eq!(*map.get(&PlayerId(1)).unwrap(), true);
    }

    #[test]
    fn test_player_map_u8() {
        let mut map = NoHashMap::default();
        map.insert(PlayerId(1), 1_u8);

        assert_eq!(*map.get(&PlayerId(1)).unwrap(), 1_u8);
    }

    #[test]
    fn test_player_map_vec() {
        let mut map = NoHashMap::default();
        map.insert(PlayerId(1), [1, 2, 3].to_vec());

        assert_eq!(*map.get(&PlayerId(1)).unwrap(), [1, 2, 3].to_vec());
    }

    #[test]
    fn test_connection_map() {
        let mut map = NoHashMap::default();
        map.insert(ConnectionId(1), [1, 2, 3].to_vec());

        assert_eq!(*map.get(&ConnectionId(1)).unwrap(), [1, 2, 3].to_vec());
    }
}
