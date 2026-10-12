/** A column of `DataGrid`. */
export interface Column<R> {
  label: string;
  /** CSS grid track size, e.g. `60px` or `minmax(160px, 2fr)`. */
  width?: string;
  align?: 'left' | 'right' | 'center';
  value: (row: R, index: number) => string | number | null | undefined;
}
