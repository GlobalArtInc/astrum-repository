import type { RepositoryType } from '../../../api';
import type { SearchComponent } from '../types';

export const PROMOTE_LIMIT = 100;

export const CONFLICT_MODES = ['fail', 'skip', 'replace'] as const;
export type ConflictMode = (typeof CONFLICT_MODES)[number];

export type PromotionStatus =
  | 'copied'
  | 'unchanged'
  | 'skipped'
  | 'conflict'
  | 'forbidden'
  | 'notFound'
  | 'unsupported'
  | 'failed';

export interface PromotionTarget {
  name: string;
  format: string;
  type: RepositoryType;
}

export interface PromotionItem {
  componentId?: string;
  image?: string;
  tag?: string;
  targetTags?: string[];
}

export interface PromotionResult {
  item?: PromotionItem;
  status: PromotionStatus;
  assetCount?: number;
  bytesCopied?: number;
  bytesReused?: number;
  skippedChildren?: number;
  error?: string;
  warning?: string;
  retryable?: boolean;
  sourceKept?: boolean;
}

export interface PromotionRequest {
  source: string;
  target: string;
  items: PromotionItem[];
  move: boolean;
  conflict: ConflictMode;
  dryRun: boolean;
}

export interface PromotionResponse {
  dryRun: boolean;
  results: PromotionResult[];
}

export interface PromotionChoice {
  target: string;
  move: boolean;
  conflict: ConflictMode;
  tags: string;
}

export function isConflictMode(value: string): value is ConflictMode {
  return (CONFLICT_MODES as readonly string[]).includes(value);
}

export function promotionItem(
  format: string,
  component: Pick<SearchComponent, 'id' | 'name' | 'version'>,
): PromotionItem {
  return format === 'docker'
    ? { image: component.name, tag: component.version }
    : { componentId: component.id };
}

export function itemLabel(item: PromotionItem | undefined): string {
  if (item === undefined) {
    return '';
  }
  return item.image === undefined ? (item.componentId ?? '') : `${item.image}:${item.tag ?? ''}`;
}

export function parseTags(text: string): string[] {
  return text
    .split(/[\s,]+/)
    .map((tag) => tag.trim())
    .filter((tag) => tag !== '');
}

export function promotionRequest(
  source: string,
  items: PromotionItem[],
  choice: PromotionChoice,
  dryRun: boolean,
): PromotionRequest {
  const tags = parseTags(choice.tags);
  return {
    source,
    target: choice.target,
    items:
      tags.length === 0
        ? items
        : items.map((item) => (item.image === undefined ? item : { ...item, targetTags: tags })),
    move: choice.move,
    conflict: choice.conflict,
    dryRun,
  };
}
