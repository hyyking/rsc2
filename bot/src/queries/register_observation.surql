BEGIN;

INSERT INTO unit $upsert_unit ON DUPLICATE KEY UPDATE last_seen = time::now();
INSERT INTO position $upsert_position ON DUPLICATE KEY UPDATE x = $x, y = $y, z = $z;
INSERT RELATION INTO has_position $upsert_has_position;

COMMIT;