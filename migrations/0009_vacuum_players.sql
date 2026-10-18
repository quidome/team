-- no-transaction
-- Writes the table file of the players again, without the old row versions that still held plain
-- names (release 2 of the encrypted player names). VACUUM cannot run inside a transaction.
VACUUM FULL players;
