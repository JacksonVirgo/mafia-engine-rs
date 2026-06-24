# Archive Planning

## General Overview

- Dedicated channels and/or categories that the bot archives all messages/edits/deletions for.
- Losing as few messages as reasonably possible to allow a perfect or near-perfect recount of what happened in those channels.

### Gathering Data

1. Live Gateway Capture

Tag: `source=gateway`

Subscribe to:
- `MESSAGE_CREATE`, `_UPDATE_`, `_DELETE`, `_DELETE_BULK`
- `MESSAGE_REACTION_ADD`, `_REMOVE`, `_REMOVE_ALL`, `_REMOVE_EMOJI`
- `THREAD_CREATE`, `_UPDATE`, `_DELETE`, `_LIST_SYNC`
- `CHANNEL_CREATE`, `_DELETE`

- Check if anything is referencing a channel/category that is to be archived, if so upsert it, otherwise drop.
- Edits append to an edit table before overwriting the message itself.
- Deletions set `deleted_at` instead of removing the row. If the `create` event was never seen, insert an empty post with `deleted_at` set.
- Insert a row on add, set `removed_at` on removal rather than deleting the row so that the reaction timeline is queryable.

If the connection drops, applicable events may be missed. Bursts also could occasionally drop events.

2. Shallow forward sweeping (burst)

Tag: `source=shallow`

- Decide on a period of time to make shallow sweeps (e.g. every 10 minutes) and track when it has been triggered for each channel.
- Call the Discord API for messages after the latest archived message ID.
    - If the result has the maximum amount of items, make an additional query to catch the rest (if it exists) and so on.
- Reconcile against the database
    - IDs from discord and not on the database are inserted.
    - IDs on the database but not on the discord have been deleted between gateway events so set `deleted_at`
- Update the time when the sweep last happened to the current time.

This should catch anything missed via dropped events and/or disconnects but as it only checks forward it may still miss messages, especially if the channel is being watched AFTER its creation.

3. Deep Sweep (continuous)

Tag: `source=deep`

A single dedicated background task that walks one page at a time, round-robined across every archivable channel. This powers ongoing verification and on-demand backfilling because they are both the same operation with a different cursor.


It should loop like the following:
- Pick the next channel
- Find where the oldest message ID from the previous *deep sweep* (or start from the newest message if none is supplied)
- Fetch the previous 100 posts from that message ID and insert if its missing.
- Store the message ID of the last message on that page (for the channel).
- Handle rate limiting/sleeping to make it not resource intensive if necessary.


The next channel should be picked based on (highest priority first):
- Channels that are marked as a priority, and whose walk hasn't reached the channels beginning.
- Channels whose last message ID is the oldest.

Stopping conditions:
- If the channel is designated for `forward_only`, stop walking once you meet that message ID
- If the channel is designated for `full`, stop when a page returns fewer than 100 messages.
- After stopping, periodically restart from the newest archived ID to re-verify (e.g. rewalk every ~24 hours) so we'd still catch any message that vanished from the database but exists on Discord, or any missed deletion events.

The cursor for the deep sweep is persisted after every page, so a crash/restart resumes mid-walk with at most one duplicate page. Make sure to check.

The walk should be slow, (e.g. one request per 3s globally, regardless of how many channels). It doesn't matter how long it takes to eventually catch up.

4. Priority Deep Sweep

Specific channels may be needed to be archived first (e.g. those that are currently subject to deletion), so a `/archive priority #channel` command exists which will:
- Set the priority on the channel.
- Clear the last deep sweep message ID so that walk restarts from the newest message.
- The channel jumps to the front of the deep-sweep priority queue.

The same logic walks it backward until the channels beginning, and from then on its treated like any other archived channel.


5. Attachments

Seperate from the message workers because attachment downloads are slow and shouldn't block message ingestion.

- When any process above attempts to archive a message that has attachments, it also inserts rows into the `archived_attachments` table with `storage_status='pending'` or equivalent and pushes their IDs onto a mpsc queue.
- A pool of workers drain the queue. For each:
    - If the environment is local, set `storage_status='disabled'`
    - Otherwise, HTTP GET the URL of the attachment. If the file size is larger than a pre-determined file size, set the status to `too_large` or equivalent. If you receive a 404/403 mark it as `failed`. Otherwise stream to a bucket at `attachments/{channel_id}/{message_id}/{attachment_id}/{filename}` and mark that status as `uploaded` with the appropriate storage key.
- Failed downloads get one retry on the next bot startup (rows still pending are re-queued by setting `attempted` or equivalent), before being set as `failed`

6. Thread / Forum Channels

TBD I can't be bothered deciding how to do this yet.

### Commands

- `/archive priority #channel`
    - Sets the channel as a priority for the deep sweep.
- `/archive status #channel`
    - Check the archiving progress.
