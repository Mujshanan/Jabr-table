/** Versioned wire contract shared by Jabr-table and Jabr-UI. */
export const JABR_DATA_SCHEMA_VERSION = 1 as const;

export type JabrErrorValue = { code: string; message: string };

export type JabrCellValue =
  | { kind: 'empty' }
  | { kind: 'boolean'; value: boolean }
  | { kind: 'integer'; value: number }
  | { kind: 'decimal'; value: string }
  | { kind: 'text'; value: string }
  | { kind: 'date_time'; value: string }
  | { kind: 'error'; value: JabrErrorValue }
  | { kind: 'list'; value: JabrCellValue[] };

export type JabrColumnType =
  | 'any' | 'boolean' | 'integer' | 'decimal' | 'text' | 'date_time';

export interface JabrColumn {
  id: string;
  title: string;
  value_type: JabrColumnType;
  unit_hint?: string;
}

export interface JabrRow {
  id: string;
  label?: string;
  cells: JabrCellValue[];
}

export interface JabrTable {
  schema_version: number;
  id: string;
  columns: JabrColumn[];
  rows: JabrRow[];
}

export interface JabrVector {
  schema_version: number;
  id: string;
  orientation: 'row' | 'column';
  values: JabrCellValue[];
  unit_hint?: string;
}

export interface JabrMatrix {
  schema_version: number;
  id: string;
  rows: number;
  columns: number;
  values: JabrCellValue[];
  row_labels: string[];
  column_labels: string[];
}

export interface JabrList {
  schema_version: number;
  id: string;
  values: JabrCellValue[];
}

export type JabrDataPayload =
  | { kind: 'table'; data: JabrTable }
  | { kind: 'vector'; data: JabrVector }
  | { kind: 'matrix'; data: JabrMatrix }
  | { kind: 'list'; data: JabrList };

export function assertSupportedSchemaVersion(version: number): void {
  if (version !== JABR_DATA_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported Jabr data schema version: ${version}; expected ${JABR_DATA_SCHEMA_VERSION}`,
    );
  }
}
