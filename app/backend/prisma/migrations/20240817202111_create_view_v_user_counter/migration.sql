-- This is an empty migration.
CREATE VIEW
  VUserCounter AS
SELECT
  FLOOR((count) / POWER(10, FLOOR(LOG (10, count)))) * POWER(10, FLOOR(LOG (10, count))) AS count_range,
  COUNT(*) AS range_count
FROM
  SessionCounter
GROUP BY
  count_range
ORDER BY
  MIN(count);
