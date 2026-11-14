#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const SOURCE = 'https://raw.githubusercontent.com/spdx/license-list-data/main/json/licenses.json';
const TARGET = path.join(ROOT, 'crates/astrum-policy/src/license/spdx.json');

const EQUIVALENTS = {
  'AGPL-1.0': 'AGPL-1.0-only',
  'AGPL-3.0': 'AGPL-3.0-only',
  'BSD-2-Clause-FreeBSD': 'BSD-2-Clause',
  'BSD-2-Clause-NetBSD': 'BSD-2-Clause',
  'bzip2-1.0.5': 'bzip2-1.0.6',
  'eCos-2.0': 'GPL-2.0-only',
  'GFDL-1.1': 'GFDL-1.1-only',
  'GFDL-1.2': 'GFDL-1.2-only',
  'GFDL-1.3': 'GFDL-1.3-only',
  'GPL-1.0': 'GPL-1.0-only',
  'GPL-1.0+': 'GPL-1.0-or-later',
  'GPL-2.0': 'GPL-2.0-only',
  'GPL-2.0+': 'GPL-2.0-or-later',
  'GPL-2.0-with-autoconf-exception': 'GPL-2.0-only',
  'GPL-2.0-with-bison-exception': 'GPL-2.0-only',
  'GPL-2.0-with-classpath-exception': 'GPL-2.0-only',
  'GPL-2.0-with-font-exception': 'GPL-2.0-only',
  'GPL-2.0-with-GCC-exception': 'GPL-2.0-only',
  'GPL-3.0': 'GPL-3.0-only',
  'GPL-3.0+': 'GPL-3.0-or-later',
  'GPL-3.0-with-autoconf-exception': 'GPL-3.0-only',
  'GPL-3.0-with-GCC-exception': 'GPL-3.0-only',
  'LGPL-2.0': 'LGPL-2.0-only',
  'LGPL-2.0+': 'LGPL-2.0-or-later',
  'LGPL-2.1': 'LGPL-2.1-only',
  'LGPL-2.1+': 'LGPL-2.1-or-later',
  'LGPL-3.0': 'LGPL-3.0-only',
  'LGPL-3.0+': 'LGPL-3.0-or-later',
  'Net-SNMP': '',
  'Nunit': '',
  'StandardML-NJ': 'SMLNJ',
  'wxWindows': 'LGPL-2.0-or-later',
};

const response = await fetch(SOURCE);
if (!response.ok) {
  throw new Error(`${SOURCE} answered ${response.status}`);
}
const official = (await response.json()).licenses;
const modern = new Set(official.filter((license) => !license.isDeprecatedLicenseId).map((license) => license.licenseId));

const entries = official.map((license) => {
  const id = license.licenseId;
  const entry = { id, name: license.name, deprecated: license.isDeprecatedLicenseId };
  if (!entry.deprecated) {
    return entry;
  }
  if (!Object.hasOwn(EQUIVALENTS, id)) {
    throw new Error(`no equivalent is written down for the deprecated id ${id}`);
  }
  const equivalent = EQUIVALENTS[id];
  if (equivalent !== '' && !modern.has(equivalent)) {
    throw new Error(`the equivalent ${equivalent} of ${id} is not a listed id`);
  }
  return { ...entry, equivalent };
});

fs.writeFileSync(TARGET, `${JSON.stringify(entries, null, 2)}\n`);
console.log(`${entries.length} ids written to ${path.relative(ROOT, TARGET)}`);
