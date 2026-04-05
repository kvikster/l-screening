import fs from 'fs';
import { nestedToFlat } from '@json-render/core';

const json = JSON.parse(fs.readFileSync('/tmp/spec_output.json', 'utf8'));
const flat = nestedToFlat(json);
console.log(JSON.stringify(flat, null, 2).slice(0, 500));
