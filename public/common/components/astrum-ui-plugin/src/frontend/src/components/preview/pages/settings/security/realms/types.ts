

/**
 * Realm data model - a security realm that can be activated
 */
export interface Realm {
  id: string;
  name: string;
}

/**
 * Props for RealmsPage component
 */
export interface RealmsPageProps {
  className?: string;
}

/**
 * Props for RealmsList component
 */
export interface RealmsListProps {
  availableRealms: Realm[];
  activeRealms: Realm[];
  onActiveRealmsChange: (realms: Realm[]) => void;
  disabled?: boolean;
}



