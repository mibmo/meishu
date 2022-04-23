ALTER TABLE scores ADD pending bool;
ALTER TABLE scores ALTER COLUMN username DROP NOT NULL;

CREATE OR REPLACE FUNCTION update_pending_status()
   RETURNS trigger AS
 $$
 BEGIN
     UPDATE scores
     SET pending = true
     WHERE username IS null AND pending IS null;
     UPDATE scores
     SET pending = false
     WHERE username IS NOT null AND pending IS null;

 RETURN NEW;
 END;

 $$
 LANGUAGE 'plpgsql';

CREATE TRIGGER pending_check
AFTER INSERT OR UPDATE
ON scores
FOR EACH ROW
EXECUTE PROCEDURE update_pending_status();
