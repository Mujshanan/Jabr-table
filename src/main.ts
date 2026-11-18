import { LitElement, css, html } from 'lit';
import './styles.css';

type ColumnType = 'text' | 'number' | 'boolean';
type Column = { id: string; name: string; type: ColumnType };
type Row = { id: string; values: Record<string, string> };
type SelectedCell = { rowId: string; columnId: string };

const initialColumns: Column[] = [
  { id: 'mark', name: 'Mark', type: 'text' },
  { id: 'section', name: 'Section', type: 'text' },
  { id: 'family', name: 'Family', type: 'text' },
  { id: 'mass', name: 'Mass (kg/m)', type: 'number' },
  { id: 'area', name: 'Area (mm²)', type: 'number' },
  { id: 'ix', name: 'Ix (cm⁴)', type: 'number' },
  { id: 'iy', name: 'Iy (cm⁴)', type: 'number' },
];

const initialRows: Row[] = [
  { id: 'row-1', values: { mark: 'UB 305×165×40', section: '305×165×40', family: 'Universal Beam', mass: '40.3', area: '51.3', ix: '8190', iy: '628' } },
  { id: 'row-2', values: { mark: 'UB 356×171×51', section: '356×171×51', family: 'Universal Beam', mass: '51.0', area: '65.0', ix: '15300', iy: '1060' } },
  { id: 'row-3', values: { mark: 'UB 406×178×60', section: '406×178×60', family: 'Universal Beam', mass: '60.1', area: '76.6', ix: '21800', iy: '1270' } },
  { id: 'row-4', values: { mark: 'UC 152×152×23', section: '152×152×23', family: 'Universal Column', mass: '23.0', area: '29.3', ix: '1260', iy: '421' } },
  { id: 'row-5', values: { mark: 'UC 203×203×46', section: '203×203×46', family: 'Universal Column', mass: '46.1', area: '58.7', ix: '4480', iy: '1510' } },
  { id: 'row-6', values: { mark: 'PFC 150×75×18', section: '150×75×18', family: 'Parallel Flange Channel', mass: '18.0', area: '22.9', ix: '870', iy: '62.3' } },
  { id: 'row-7', values: { mark: 'RHS 100×50×5', section: '100×50×5', family: 'Rectangular Hollow', mass: '10.9', area: '13.9', ix: '184', iy: '80.8' } },
  { id: 'row-8', values: { mark: 'SHS 100×100×5', section: '100×100×5', family: 'Square Hollow', mass: '14.7', area: '18.7', ix: '264', iy: '264' } },
];

function columnLetter(index: number): string {
  let value = index + 1;
  let label = '';
  while (value > 0) {
    const remainder = (value - 1) % 26;
    label = String.fromCharCode(65 + remainder) + label;
    value = Math.floor((value - 1) / 26);
  }
  return label;
}

function parseDelimited(text: string, delimiter: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = '';
  let quoted = false;

  for (let i = 0; i < text.length; i += 1) {
    const char = text[i];
    if (char === '"' && quoted && text[i + 1] === '"') {
      field += '"';
      i += 1;
    } else if (char === '"') {
      quoted = !quoted;
    } else if (char === delimiter && !quoted) {
      row.push(field);
      field = '';
    } else if ((char === '\n' || char === '\r') && !quoted) {
      if (char === '\r' && text[i + 1] === '\n') i += 1;
      row.push(field);
      if (row.some((value) => value.length > 0)) rows.push(row);
      row = [];
      field = '';
    } else {
      field += char;
    }
  }

  row.push(field);
  if (row.some((value) => value.length > 0)) rows.push(row);
  return rows;
}

function escapeCsv(value: string): string {
  return /[",\r\n]/.test(value) ? '"' + value.replaceAll('"', '""') + '"' : value;
}

class JabrTableApp extends LitElement {
  static properties = {
    view: { state: true },
    columns: { state: true },
    rows: { state: true },
    selectedCell: { state: true },
    search: { state: true },
    sortColumn: { state: true },
    sortDirection: { state: true },
    status: { state: true },
  };

  static styles = css`
    :host { display: block; min-height: 100vh; color: #172033; }
    * { box-sizing: border-box; }
    button, input { font: inherit; }
    button { cursor: pointer; }
    .shell { min-height: 100vh; display: grid; grid-template-columns: 232px minmax(0, 1fr); }
    .sidebar { background: #f8fafc; border-right: 1px solid #e5eaf1; padding: 22px 14px; display: flex; flex-direction: column; gap: 28px; }
    .brand { display: flex; align-items: center; gap: 10px; padding: 0 9px; font-weight: 750; letter-spacing: -.04em; font-size: 19px; }
    .brand-mark { width: 30px; height: 30px; border-radius: 9px; background: #1e3a5f; color: white; display: grid; place-items: center; font-size: 16px; }
    .side-label { color: #94a0b2; text-transform: uppercase; font-size: 10px; font-weight: 750; letter-spacing: .11em; padding: 0 10px; margin-bottom: 8px; }
    .side-link { width: 100%; display: flex; gap: 10px; align-items: center; padding: 9px 10px; border: 0; background: transparent; color: #64748b; border-radius: 8px; text-align: left; font-size: 13px; }
    .side-link.active { background: #eaf0f7; color: #173b63; font-weight: 650; }
    .side-icon { width: 17px; text-align: center; color: #7b8ba1; }
    .workspace-note { margin-top: auto; border: 1px solid #e4eaf1; background: white; border-radius: 10px; padding: 12px; color: #718096; font-size: 11px; line-height: 1.55; }
    .workspace-note strong { color: #334155; display: block; margin-bottom: 3px; font-size: 12px; }
    .main { min-width: 0; display: flex; flex-direction: column; }
    .topbar { height: 58px; display: flex; align-items: center; justify-content: space-between; padding: 0 28px; border-bottom: 1px solid #e8edf3; background: rgba(255,255,255,.9); }
    .crumb { color: #8b98aa; font-size: 12px; }
    .crumb strong { color: #43536a; font-weight: 600; }
    .top-actions { display: flex; align-items: center; gap: 9px; color: #718096; font-size: 12px; }
    .avatar { width: 27px; height: 27px; display: grid; place-items: center; border-radius: 50%; background: #e9eff7; color: #31577e; font-size: 10px; font-weight: 700; }
    .content { padding: 30px 32px 20px; width: 100%; max-width: 1600px; margin: 0 auto; }
    .heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; margin-bottom: 23px; }
    .eyebrow { color: #8b98aa; font-size: 11px; text-transform: uppercase; font-weight: 700; letter-spacing: .1em; margin-bottom: 8px; }
    h1 { margin: 0; font-size: 25px; line-height: 1.2; letter-spacing: -.045em; color: #16243a; font-weight: 720; }
    .subtitle { margin: 7px 0 0; color: #8491a3; font-size: 13px; }
    .heading-actions { display: flex; align-items: center; gap: 8px; }
    .button { border: 1px solid #dce3eb; color: #46566b; background: white; border-radius: 7px; padding: 8px 12px; font-size: 12px; font-weight: 600; display: inline-flex; align-items: center; gap: 7px; }
    .button:hover { border-color: #aebdce; background: #fbfcfe; }
    .button.primary { background: #1e426b; border-color: #1e426b; color: white; }
    .button.primary:hover { background: #173756; }
    .button:focus-visible, input:focus-visible { outline: 2px solid #6d9ccc; outline-offset: 2px; }
    .tabs-line { display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #e4eaf1; margin-bottom: 0; }
    .tabs { display: flex; align-items: center; gap: 20px; }
    .tab { border: 0; background: transparent; padding: 12px 2px 11px; color: #8491a3; font-size: 12px; border-bottom: 2px solid transparent; }
    .tab.active { color: #23496f; font-weight: 700; border-bottom-color: #315f8e; }
    .table-meta { display: flex; gap: 14px; color: #97a2b1; font-size: 11px; }
    .toolbar { min-height: 53px; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 0; }
    .tool-group { display: flex; align-items: center; gap: 7px; }
    .tool-button { border: 1px solid transparent; background: transparent; color: #627187; border-radius: 6px; padding: 7px 9px; font-size: 11px; }
    .tool-button:hover { border-color: #e1e7ee; background: white; color: #284c70; }
    .search { width: 210px; border: 1px solid #e0e6ed; border-radius: 7px; background: white; padding: 8px 10px; color: #34445a; font-size: 11px; }
    .grid-card { border: 1px solid #e1e7ee; background: white; border-radius: 9px; overflow: hidden; box-shadow: 0 2px 7px rgba(25,42,62,.025); }
    .formula-bar { display: flex; align-items: center; min-height: 37px; border-bottom: 1px solid #e8edf2; background: #fbfcfd; padding: 5px 10px; gap: 10px; }
    .cell-ref { min-width: 48px; border-right: 1px solid #e2e8ef; padding: 4px 10px 4px 3px; color: #4d6077; font: 600 11px ui-monospace, SFMono-Regular, Menlo, monospace; }
    .fx { color: #98a4b3; font: italic 600 12px Georgia, serif; }
    .formula-value { color: #718096; font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .table-scroll { overflow: auto; max-height: calc(100vh - 355px); min-height: 320px; }
    table { width: 100%; border-collapse: separate; border-spacing: 0; table-layout: fixed; font-size: 11px; }
    th { position: sticky; top: 0; z-index: 2; background: #f7f9fb; color: #718096; font-weight: 650; text-align: left; border-bottom: 1px solid #e3e9ef; border-right: 1px solid #edf0f4; height: 42px; padding: 5px 10px; }
    th:first-child { width: 42px; text-align: center; color: #a2adba; }
    th .col-letter { display: block; color: #a0aab7; font-size: 9px; font-weight: 500; margin-bottom: 2px; }
    th .col-name { display: block; color: #506178; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    th button { border: 0; background: none; color: inherit; text-align: left; padding: 0; width: 100%; cursor: pointer; }
    td { height: 37px; border-bottom: 1px solid #eef1f5; border-right: 1px solid #f0f2f5; padding: 0; color: #43536a; }
    tr:last-child td { border-bottom: 0; }
    td.row-index { text-align: center; background: #fafbfd; color: #a1adbb; font-size: 10px; }
    td input { width: 100%; height: 36px; border: 1px solid transparent; border-radius: 0; padding: 0 10px; color: #34445a; background: transparent; font-size: 11px; outline: none; }
    td input.numeric { text-align: right; font-variant-numeric: tabular-nums; }
    td input:focus { background: #f7fbff; border: 1px solid #5c8fbd; box-shadow: inset 0 0 0 1px #5c8fbd; }
    td input.selected { background: #f7fbff; }
    .database-wrap { padding: 0; }
    .db-summary { display: flex; align-items: center; justify-content: space-between; padding: 13px 14px; border-bottom: 1px solid #edf0f4; color: #8190a3; font-size: 11px; }
    .db-summary strong { color: #41536b; font-weight: 650; }
    .type-pill { display: inline-flex; align-items: center; border-radius: 4px; padding: 3px 5px; font-size: 9px; font-weight: 650; background: #eef3f8; color: #647991; margin-left: 6px; }
    .type-pill.number { background: #eef6f3; color: #53806d; }
    .record-cell { padding: 0 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .record-cell.primary { color: #315c84; font-weight: 650; }
    .record-id { color: #a0aaba; font: 10px ui-monospace, monospace; }
    .empty { padding: 40px; text-align: center; color: #8794a6; }
    .statusbar { display: flex; justify-content: space-between; align-items: center; padding: 12px 2px; color: #98a3b1; font-size: 10px; }
    .status-left { display: flex; gap: 14px; align-items: center; }
    .status-dot { width: 6px; height: 6px; border-radius: 50%; background: #78a88e; display: inline-block; margin-right: 5px; }
    @media (max-width: 900px) {
      .shell { grid-template-columns: 62px minmax(0,1fr); }
      .sidebar { padding: 18px 8px; align-items: center; }
      .brand { padding: 0; }
      .brand-word, .side-label, .side-link span:not(.side-icon), .workspace-note { display: none; }
      .side-link { justify-content: center; padding: 10px 5px; }
      .content { padding: 22px 16px; }
      .topbar { padding: 0 16px; }
      .table-scroll { max-height: calc(100vh - 380px); }
    }
    @media (max-width: 600px) {
      .heading { flex-direction: column; }
      .heading-actions { width: 100%; }
      .toolbar { align-items: flex-start; flex-direction: column; }
      .search { width: 100%; }
      .heading-actions .button { flex: 1; justify-content: center; }
      .table-meta { display: none; }
    }
  `;

  view: 'table' | 'database' = 'table';
  columns: Column[] = [...initialColumns];
  rows: Row[] = [...initialRows];
  selectedCell: SelectedCell = { rowId: 'row-1', columnId: 'mark' };
  search = '';
  sortColumn: string | null = null;
  sortDirection: 'asc' | 'desc' = 'asc';
  status = 'All changes are local to this prototype';

  get visibleRows(): Row[] {
    const needle = this.search.trim().toLowerCase();
    let result = this.rows.filter((row) => !needle || this.columns.some((column) => (row.values[column.id] ?? '').toLowerCase().includes(needle)));
    if (this.sortColumn) {
      const key = this.sortColumn;
      const direction = this.sortDirection === 'asc' ? 1 : -1;
      result = [...result].sort((a, b) => {
        const av = a.values[key] ?? '';
        const bv = b.values[key] ?? '';
        const an = Number(av);
        const bn = Number(bv);
        const bothNumeric = av.trim() !== '' && bv.trim() !== '' && Number.isFinite(an) && Number.isFinite(bn);
        return (bothNumeric ? an - bn : av.localeCompare(bv, undefined, { numeric: true, sensitivity: 'base' })) * direction;
      });
    }
    return result;
  }

  get selectedValue(): string {
    return this.rows.find((row) => row.id === this.selectedCell.rowId)?.values[this.selectedCell.columnId] ?? '';
  }

  get selectedReference(): string {
    const rowIndex = this.rows.findIndex((row) => row.id === this.selectedCell.rowId);
    const columnIndex = this.columns.findIndex((column) => column.id === this.selectedCell.columnId);
    return rowIndex >= 0 && columnIndex >= 0 ? columnLetter(columnIndex) + (rowIndex + 1) : 'A1';
  }

  private selectCell(rowId: string, columnId: string): void {
    this.selectedCell = { rowId, columnId };
  }

  private updateCell(rowId: string, columnId: string, value: string): void {
    this.rows = this.rows.map((row) => row.id === rowId ? { ...row, values: { ...row.values, [columnId]: value } } : row);
    this.selectedCell = { rowId, columnId };
    this.status = 'Edited cell ' + this.selectedReference;
  }

  private addRow(): void {
    const id = 'row-' + crypto.randomUUID();
    const values = Object.fromEntries(this.columns.map((column) => [column.id, '']));
    this.rows = [...this.rows, { id, values }];
    this.selectedCell = { rowId: id, columnId: this.columns[0]?.id ?? '' };
    this.status = 'Added a new record';
  }

  private addColumn(): void {
    const index = this.columns.length + 1;
    const id = 'field-' + crypto.randomUUID();
    this.columns = [...this.columns, { id, name: 'New field ' + index, type: 'text' }];
    this.rows = this.rows.map((row) => ({ ...row, values: { ...row.values, [id]: '' } }));
    this.status = 'Added a new column';
  }

  private sortBy(columnId: string): void {
    if (this.sortColumn === columnId) this.sortDirection = this.sortDirection === 'asc' ? 'desc' : 'asc';
    else { this.sortColumn = columnId; this.sortDirection = 'asc'; }
  }

  private focusCell(rowIndex: number, columnIndex: number): void {
    const row = this.visibleRows[rowIndex];
    const column = this.columns[columnIndex];
    if (!row || !column) return;
    this.selectCell(row.id, column.id);
    this.updateComplete.then(() => {
      this.renderRoot.querySelector<HTMLInputElement>(`input[data-row-index="${rowIndex}"][data-column-index="${columnIndex}"]`)?.focus();
    });
  }

  private handleCellKeydown(event: KeyboardEvent, rowIndex: number, columnIndex: number): void {
    let nextRow = rowIndex;
    let nextColumn = columnIndex;
    if (event.key === 'ArrowDown') nextRow += 1;
    else if (event.key === 'ArrowUp') nextRow -= 1;
    else if (event.key === 'Tab') { nextColumn += event.shiftKey ? -1 : 1; if (nextColumn >= this.columns.length) { nextColumn = 0; nextRow += 1; } if (nextColumn < 0) { nextColumn = this.columns.length - 1; nextRow -= 1; } }
    else if (event.key === 'Enter') nextRow += 1;
    else return;
    event.preventDefault();
    this.focusCell(Math.max(0, Math.min(this.visibleRows.length - 1, nextRow)), Math.max(0, Math.min(this.columns.length - 1, nextColumn)));
  }

  private handlePaste(event: ClipboardEvent): void {
    const text = event.clipboardData?.getData('text/plain');
    if (!text || !/[\t\n]/.test(text)) return;
    event.preventDefault();
    const matrix = parseDelimited(text, '\t');
    const startRow = this.visibleRows.findIndex((row) => row.id === this.selectedCell.rowId);
    const startColumn = this.columns.findIndex((column) => column.id === this.selectedCell.columnId);
    if (startRow < 0 || startColumn < 0) return;
    const nextRows = [...this.rows];
    for (let r = 0; r < matrix.length; r += 1) {
      const targetRow = startRow + r;
      if (!nextRows[targetRow]) {
        nextRows.push({ id: 'row-' + crypto.randomUUID(), values: Object.fromEntries(this.columns.map((column) => [column.id, ''])) });
      }
      const row = nextRows[targetRow];
      const values = { ...row.values };
      for (let c = 0; c < matrix[r].length && startColumn + c < this.columns.length; c += 1) values[this.columns[startColumn + c].id] = matrix[r][c];
      nextRows[targetRow] = { ...row, values };
    }
    this.rows = nextRows;
    this.status = 'Pasted ' + matrix.length + ' row(s)';
  }

  private exportCsv(): void {
    const header = this.columns.map((column) => escapeCsv(column.name)).join(',');
    const body = this.visibleRows.map((row) => this.columns.map((column) => escapeCsv(row.values[column.id] ?? '')).join(','));
    const blob = new Blob([[header, ...body].join('\r\n')], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'jabr-table-export.csv';
    anchor.click();
    URL.revokeObjectURL(url);
    this.status = 'CSV export prepared';
  }

  private async importCsv(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const parsed = parseDelimited(await file.text(), ',');
      if (parsed.length < 1) throw new Error('The file contains no rows');
      const headers = parsed[0].map((name, index) => name.trim() || 'Column ' + (index + 1));
      const columns = headers.map((name, index) => ({ id: 'import-' + index + '-' + crypto.randomUUID(), name, type: 'text' as ColumnType }));
      const rows = parsed.slice(1).map((values) => ({
        id: 'row-' + crypto.randomUUID(),
        values: Object.fromEntries(columns.map((column, index) => [column.id, values[index] ?? ''])),
      }));
      this.columns = columns;
      this.rows = rows;
      this.selectedCell = { rowId: rows[0]?.id ?? '', columnId: columns[0]?.id ?? '' };
      this.status = 'Imported ' + rows.length + ' record(s) from ' + file.name;
    } catch (error) {
      this.status = error instanceof Error ? 'Import failed: ' + error.message : 'Import failed';
    } finally {
      input.value = '';
    }
  }

  render() {
    const rows = this.visibleRows;
    return html`
      <div class="shell">
        <aside class="sidebar">
          <div class="brand"><span class="brand-mark">J</span><span class="brand-word">jabr<span style="color:#8fa2b8">/</span>table</span></div>
          <div>
            <div class="side-label">Workspace</div>
            <button class="side-link active"><span class="side-icon">▦</span><span>Data tables</span></button>
            <button class="side-link"><span class="side-icon">⌘</span><span>Recent</span></button>
          </div>
          <div>
            <div class="side-label">Current table</div>
            <button class="side-link active"><span class="side-icon">▤</span><span>Steel sections</span></button>
            <button class="side-link"><span class="side-icon">＋</span><span>New table</span></button>
          </div>
          <div class="workspace-note"><strong>Engineering workspace</strong>Prototype data is stored in memory. Persistence and collaboration are planned for the document integration stage.</div>
        </aside>

        <main class="main">
          <header class="topbar">
            <div class="crumb">Workspace <span style="padding:0 7px;color:#c0c8d2">/</span> Data tables <span style="padding:0 7px;color:#c0c8d2">/</span> <strong>Steel sections</strong></div>
            <div class="top-actions"><span>Local prototype</span><span class="avatar">JS</span></div>
          </header>

          <section class="content">
            <div class="heading">
              <div>
                <div class="eyebrow">Engineering data</div>
                <h1>Steel section catalogue</h1>
                <p class="subtitle">A shared table for engineering data entry, lookup and structured records.</p>
              </div>
              <div class="heading-actions">
                <button class="button" @click=${() => this.exportCsv()}>↓ Export CSV</button>
                <label class="button" for="csv-import">↑ Import CSV</label>
                <input id="csv-import" type="file" accept=".csv,text/csv" hidden @change=${(event: Event) => this.importCsv(event)} />
                <button class="button primary" @click=${() => this.addRow()}>＋ Add record</button>
              </div>
            </div>

            <div class="tabs-line">
              <div class="tabs">
                <button class="tab ${this.view === 'table' ? 'active' : ''}" @click=${() => this.view = 'table'}>▦ Engineering table</button>
                <button class="tab ${this.view === 'database' ? 'active' : ''}" @click=${() => this.view = 'database'}>▤ Database view</button>
              </div>
              <div class="table-meta"><span>● Saved locally</span><span>${this.columns.length} fields</span><span>${this.rows.length} records</span></div>
            </div>

            <div class="toolbar">
              <div class="tool-group">
                <button class="tool-button" @click=${() => this.addRow()}>＋ Row</button>
                <button class="tool-button" @click=${() => this.addColumn()}>＋ Column</button>
                <span style="height:18px;border-left:1px solid #e5eaf0;margin:0 3px"></span>
                <button class="tool-button" @click=${() => { this.search = ''; }}>Clear search</button>
              </div>
              <div class="tool-group">
                <input class="search" type="search" placeholder="Search records…" .value=${this.search} @input=${(event: InputEvent) => this.search = (event.target as HTMLInputElement).value} />
              </div>
            </div>

            <div class="grid-card" @paste=${(event: ClipboardEvent) => this.handlePaste(event)}>
              ${this.view === 'table' ? this.renderGrid(rows) : this.renderDatabase(rows)}
            </div>

            <div class="statusbar">
              <div class="status-left"><span><i class="status-dot"></i>Prototype</span><span>${this.status}</span></div>
              <div>${rows.length} of ${this.rows.length} records</div>
            </div>
          </section>
        </main>
      </div>
    `;
  }

  private renderGrid(rows: Row[]) {
    return html`
      <div class="formula-bar">
        <span class="cell-ref">${this.selectedReference}</span>
        <span class="fx">fx</span>
        <span class="formula-value">${this.selectedValue || 'Select a cell to inspect its value'}</span>
      </div>
      <div class="table-scroll">
        <table>
          <thead><tr>
            <th>#</th>
            ${this.columns.map((column, index) => html`<th><button @click=${() => this.sortBy(column.id)}><span class="col-letter">${columnLetter(index)} ${this.sortColumn === column.id ? (this.sortDirection === 'asc' ? '↑' : '↓') : ''}</span><span class="col-name">${column.name}</span></button></th>`)}
          </tr></thead>
          <tbody>
            ${rows.length ? rows.map((row, rowIndex) => html`
              <tr>
                <td class="row-index">${rowIndex + 1}</td>
                ${this.columns.map((column, columnIndex) => html`
                  <td><input
                    class="${column.type === 'number' ? 'numeric' : ''} ${this.selectedCell.rowId === row.id && this.selectedCell.columnId === column.id ? 'selected' : ''}"
                    aria-label="${column.name}, row ${rowIndex + 1}"
                    data-row-index="${rowIndex}"
                    data-column-index="${columnIndex}"
                    .value=${row.values[column.id] ?? ''}
                    @focus=${() => this.selectCell(row.id, column.id)}
                    @input=${(event: InputEvent) => this.updateCell(row.id, column.id, (event.target as HTMLInputElement).value)}
                    @keydown=${(event: KeyboardEvent) => this.handleCellKeydown(event, rowIndex, columnIndex)}
                  /></td>
                `)}
              </tr>
            `) : html`<tr><td colspan=${this.columns.length + 1}><div class="empty">No records match your search.</div></td></tr>`}
          </tbody>
        </table>
      </div>
    `;
  }

  private renderDatabase(rows: Row[]) {
    return html`
      <div class="db-summary"><span><strong>All records</strong> · Structured database view</span><span>Sort by selecting a column heading</span></div>
      <div class="table-scroll">
        <table>
          <thead><tr>
            <th>#</th>
            ${this.columns.map((column) => html`<th><button @click=${() => this.sortBy(column.id)}><span class="col-name">${column.name} ${this.sortColumn === column.id ? (this.sortDirection === 'asc' ? '↑' : '↓') : ''}</span><span class="type-pill ${column.type}">${column.type}</span></button></th>`)}
          </tr></thead>
          <tbody>
            ${rows.length ? rows.map((row, rowIndex) => html`
              <tr>
                <td class="row-index">${rowIndex + 1}</td>
                ${this.columns.map((column, columnIndex) => html`
                  <td><input
                    class="${column.type === 'number' ? 'numeric' : ''} ${this.selectedCell.rowId === row.id && this.selectedCell.columnId === column.id ? 'selected' : ''}"
                    aria-label="${column.name}, record ${rowIndex + 1}"
                    data-row-index="${rowIndex}"
                    data-column-index="${columnIndex}"
                    .value=${row.values[column.id] ?? ''}
                    @focus=${() => this.selectCell(row.id, column.id)}
                    @input=${(event: InputEvent) => this.updateCell(row.id, column.id, (event.target as HTMLInputElement).value)}
                    @keydown=${(event: KeyboardEvent) => this.handleCellKeydown(event, rowIndex, columnIndex)}
                  /></td>
                `)}
              </tr>
            `) : html`<tr><td colspan=${this.columns.length + 1}><div class="empty">No records match your search.</div></td></tr>`}
          </tbody>
        </table>
      </div>
    `;
  }
}

customElements.define('jabr-table-app', JabrTableApp);
