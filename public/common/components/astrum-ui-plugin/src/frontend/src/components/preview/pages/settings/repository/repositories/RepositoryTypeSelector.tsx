

import React, { useState, useEffect, useMemo } from 'react';
import { Box, Flex, Text, Heading, Card } from '@radix-ui/themes';
import { Loader2, } from 'lucide-react';

import { SettingsAlert } from '../../../../shared/form';
import { useRepositoriesApi } from './useRepositoriesApi';
import { FormatIcon } from './components/FormatIcon';
import SearchHeader from '../../../search/unified/SearchHeader';
import { 
  Recipe, 
  RepositoryTypeSelectorProps, 
  RepositoryType, 
  FORMAT_LABELS, 
  TYPE_LABELS 
} from './types';
import { 
  getFormatDescription, 
  getTypeDescription, 
} from './repositoryTypeDescriptions';

import './RepositoryTypeSelector.scss';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';

/**
 * RepositoryTypeSelector - Multi-stage wizard for selecting technology and architecture.
 * 
 * Stage 1: Select Format (Technology)
 * Stage 2: Select Type (Hosted, Proxy, Group)
 */
export function RepositoryTypeSelector({
  onSelect,
  onCancel,
  mode,
  hideActions = false,
  selectedFormat: initialFormat,
  onFormatSelect,
  onSelectionChange
}: RepositoryTypeSelectorProps) {
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedFormat, setSelectedFormat] = useState<string | null>(initialFormat || null);
  const [selectedType, setSelectedType] = useState<RepositoryType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const { fetchRecipes } = useRepositoriesApi();

  // Load recipes on mount
  useEffect(() => {
    const loadRecipes = async () => {
      setLoading(true);
      try {
        const data = await fetchRecipes();
        setRecipes(data);
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : RepositoriesPageStrings.TYPE_SELECTOR.FAILED_LOAD_RECIPES);
      } finally {
        setLoading(false);
      }
    };
    loadRecipes();
  }, [fetchRecipes]);

  // Sync internal state with prop
  useEffect(() => {
    if (initialFormat !== undefined) {
      setSelectedFormat(initialFormat);
      if (!initialFormat) {
        setSelectedType(null);
      }
    }
  }, [initialFormat]);

  // Determine effective mode: prop mode or internal state
  const effectiveMode = mode || (selectedFormat ? 'type' : 'format');

  // Unique list of formats from recipes
  const availableFormats = useMemo(() => {
    const formats = new Set(recipes.map(r => r.format));
    let result = Array.from(formats).sort((a, b) => 
      (FORMAT_LABELS[a] || a).localeCompare(FORMAT_LABELS[b] || b)
    );

    if (searchTerm) {
      const q = searchTerm.toLowerCase();
      result = result.filter(f =>
        (FORMAT_LABELS[f] || f).toLowerCase().includes(q) ||
        f.toLowerCase().includes(q)
      );
    }
    return result;
  }, [recipes, searchTerm]);

  // Types available for the selected format
  const availableTypesForFormat = useMemo(() => {
    if (!selectedFormat) return [];
    return recipes
      .filter(r => r.format === selectedFormat)
      .map(r => r.type)
      .sort((a, b) => {
        const order = { proxy: 0, hosted: 1, group: 2 };
        return (order[a] ?? 99) - (order[b] ?? 99);
      });
  }, [selectedFormat, recipes]);

  const handleFormatClick = (format: string) => {
    setSelectedFormat(format);
    setSelectedType(null);
    if (onFormatSelect) onFormatSelect(format);
    if (onSelectionChange) {
      // Notify parent that a format is picked
      onSelectionChange(true, { format, type: null as any });
    }
  };

  const handleTypeClick = (type: RepositoryType) => {
    setSelectedType(type);
    const recipe = recipes.find(r => r.format === selectedFormat && r.type === type);
    if (recipe && onSelectionChange) {
      // Both format and type selected
      onSelectionChange(true, recipe);
    }
    // If not in external wizard control, advance now
    if (!hideActions && recipe) {
      onSelect(recipe);
    }
  };

  if (loading) {
    return (
      <Flex align="center" justify="center" p="8">
        <Loader2 className="animate-spin" />
        <Text ml="2">{RepositoriesPageStrings.TYPE_SELECTOR.LOADING_REPOSITORY_RECIPES}</Text>
      </Flex>
    );
  }

  if (error) {
    return (
      <Box p="4">
        <SettingsAlert type="error">{error}</SettingsAlert>
      </Box>
    );
  }

  // --- STAGE 2: SELECT TYPE ---
  if (effectiveMode === 'type' && selectedFormat) {
    return (
      <Box className="repository-type-selector">
        {/* Title with count (matches PageHeader pattern) */}
        <Flex align="center" gap="2" mb="4">
          <Heading as="h2" size="6" weight="bold">
            {RepositoriesPageStrings.TYPE_SELECTOR.SELECT_TYPE}
          </Heading>
          <Text size="2" color="gray">
            {availableTypesForFormat.length}
          </Text>
        </Flex>

        <Box mb="6">
          <Flex align="center" gap="3" mt={mode ? '0' : '2'}>
            <FormatIcon format={selectedFormat} size={48} />
            <Box>
              <Heading size="5">{FORMAT_LABELS[selectedFormat] || selectedFormat}</Heading>
              <Text size="2" color="gray">{getFormatDescription(selectedFormat)}</Text>
            </Box>
          </Flex>
        </Box>

        <Box className="repository-type-selector__grid repository-type-selector__grid--types">
          {availableTypesForFormat.map(type => {
            const isSelected = selectedType === type;

            return (
              <Card
                key={type}
                asChild
                className={`repository-type-selector__card${isSelected ? ' repository-type-selector__card--selected' : ''}`}
              >
                <button
                  type="button"
                  onClick={() => handleTypeClick(type as RepositoryType)}
                  aria-pressed={selectedType === type}
                >
                  <Box p="3">
                    <Text weight="bold" size="3" mb="1" style={{ display: 'block' }}>
                      {TYPE_LABELS[type] || type}
                    </Text>
                    <Text size="2" color="gray" style={{ lineHeight: '1.4' }}>
                      {getTypeDescription(type)}
                    </Text>
                  </Box>
                </button>
              </Card>
            );
          })}
        </Box>
      </Box>
    );
  }

  // --- STAGE 1: SELECT FORMAT ---
  return (
    <Box className="repository-type-selector">
      {/* Title with count (matches PageHeader pattern) */}
      <Flex align="center" gap="2" mb="4">
        <Heading as="h2" size="6" weight="bold">
          {RepositoriesPageStrings.TYPE_SELECTOR.SELECT_FORMAT}
        </Heading>
        <Text size="2" color="gray">
          {availableFormats.length}
        </Text>
      </Flex>

      <Box className="repository-type-selector__search">
        <SearchHeader
          format="all"
          onFormatChange={() => {}}
          query={searchTerm}
          onSearch={setSearchTerm}
          placeholder={RepositoriesPageStrings.TYPE_SELECTOR.SEARCH_TECHNOLOGY_E_G_DOCKER_PLACEHOLDER}
        />
      </Box>

      <Box className="repository-type-selector__grid">
        {availableFormats.map(format => {
          const isSelected = selectedFormat === format;

          return (
            <Card
              key={format}
              asChild
              className={`repository-type-selector__format-card${isSelected ? ' repository-type-selector__format-card--selected' : ''}`}
            >
              <button
                type="button"
                onClick={() => handleFormatClick(format)}
                aria-pressed={selectedFormat === format}
              >
                <Box p="3" className="repository-type-selector__format-card-content">
                  <FormatIcon format={format} size={32} />
                  <Text size="1" weight="medium">
                    {FORMAT_LABELS[format] || format}
                  </Text>
                </Box>
              </button>
            </Card>
          );
        })}
      </Box>
    </Box>
  );
}

export default RepositoryTypeSelector;
