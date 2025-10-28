CREATE TABLE feature_flags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE
);

CREATE TABLE server_flags (
    server_id BIGINT UNSIGNED NOT NULL,
    flag_id INT NOT NULL,
    PRIMARY KEY (server_id, flag_id)
);

ALTER TABLE servers ADD COLUMN player_chat_server_id BIGINT UNSIGNED;

ALTER TABLE servers ADD CONSTRAINT fk_player_chat_server FOREIGN KEY (player_chat_server_id) REFERENCES servers(id) ON DELETE SET NULL;
