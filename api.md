# Web API Specifications

This document describes Dale Web API surface for clients to use. The design architecture stands on two principles: stateless HTTP for data retrieval and WS messaging for client state invalidation.

## HTTP API endpoints

### GET /api/library/

Purpose: fetch entire library state of declared primitives.

It will include:

- All available collections + their configured filters, groupers, orders.
- All available cabinets + their shelves, orders.
- All available filters, groupers, orders registered in config.
- All available actions and their data.

### GET /api/albums/{id}

Purpose: fetch `album.lock.json` via album id.

### GET /api/view

Purpose: fetch array of albums via primitive parameters.

Parameters:
 - `collection` : one of declared `dale.collection`
 - `filter` : one of declared `dale.filter`
 - `order` : one of declared `dale.order`
 - `grouper` : one of declared `dale.grouper`
 - `group` : group value belonging to the `grouper`
