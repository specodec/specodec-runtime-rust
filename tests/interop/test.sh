#!/bin/bash
set -e
cd "$(dirname "$0")"
npm install
node test.mjs