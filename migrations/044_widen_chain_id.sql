-- Widen chain_id to BIGINT to support u32 values from WASM crypto
ALTER TABLE sender_key_distributions ALTER COLUMN chain_id TYPE BIGINT;
