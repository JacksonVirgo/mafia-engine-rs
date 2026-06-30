CREATE TABLE votecounter (
  channel_id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE votecounter_slot (
  id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
  name VARCHAR(32),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE votecounter_slot_member (
  slot_id BIGINT UNSIGNED NOT NULL,
  member_id BIGINT UNSIGNED NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (slot_id, member_id),
  FOREIGN KEY (slot_id) REFERENCES votecounter_slot (id) ON DELETE CASCADE,
  FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE
);

CREATE TABLE votecounter_event (
  id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
  channel_id BIGINT UNSIGNED NOT NULL,
  slot_id BIGINT UNSIGNED,
  /* Vote Management */
  vote_target BIGINT UNSIGNED,
  is_skipping BOOLEAN,
  is_unvoting BOOLEAN,
  vote_weight INT,
  vote_penalty INT,
  is_dead BOOLEAN,
  can_be_voted BOOLEAN,
  can_vote BOOLEAN,
  counts_for_majority BOOLEAN,
  /* VC Management */
  timed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  /* Constraints */
  FOREIGN KEY (channel_id) REFERENCES votecounter (channel_id) ON DELETE CASCADE,
  FOREIGN KEY (slot_id) REFERENCES votecounter_slot (id) ON DELETE CASCADE
);
