import { createContext, useContext } from 'react';

export interface SearchPanelContextValue {
  close(): void;
}

export const SearchPanelContext = createContext<SearchPanelContextValue>({ close: () => {} });

export function useSearchPanel(): SearchPanelContextValue {
  return useContext(SearchPanelContext);
}

export const ITEM_SELECTOR = '[data-search-item]';
