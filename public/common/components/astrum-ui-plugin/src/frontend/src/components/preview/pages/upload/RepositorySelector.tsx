

import React, { useMemo } from 'react';

import { SettingsCombobox, SettingsFormSection } from '../../shared/form';
import type { UploadableRepository } from './upload.types';

interface RepositorySelectorProps {
  repositoryName: string;
  onRepositoryChange: (repoName: string) => void;
  availableRepositories: UploadableRepository[];
  disabled?: boolean;
}

/**
 * Component for selecting a target repository for upload.
 *
 * @param {RepositorySelectorProps} props - The props for the component.
 * @returns {JSX.Element} The rendered component.
 */
export function RepositorySelector({
  repositoryName,
  onRepositoryChange,
  availableRepositories,
  disabled = false,
}: RepositorySelectorProps): JSX.Element {
  const repositoryOptions = useMemo(
    () =>
      availableRepositories.map((repo) => ({
        value: repo.name,
        label: repo.name,
        description: repo.format,
      })),
    [availableRepositories],
  );

  return (
    <SettingsFormSection title="Repository">
      <SettingsCombobox
        name="repository"
        label="Target Repository"
        value={repositoryName}
        onChange={(val) => onRepositoryChange(val)}
        options={repositoryOptions}
        helpText="The hosted repository where the component will be uploaded"
        required
        allowCustom={false}
        disabled={disabled}
        aria-label="Select target repository"
      />
    </SettingsFormSection>
  );
}

export default RepositorySelector;
