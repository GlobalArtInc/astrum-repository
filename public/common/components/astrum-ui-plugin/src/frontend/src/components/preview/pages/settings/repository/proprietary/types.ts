

/**
 * Repository reference for transfer list
 */
export interface RepositoryReference {
  id: string;
  name: string;
}

/**
 * Proprietary repositories settings data
 */
export interface ProprietaryRepositoriesSettings {
  enabledRepositories: string[];
}

/**
 * Props for ProprietaryPage component
 */
export interface ProprietaryPageProps {
  className?: string;
}


