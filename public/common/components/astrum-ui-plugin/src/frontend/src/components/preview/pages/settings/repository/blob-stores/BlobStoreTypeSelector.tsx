

import React from 'react';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import {
  HardDrive,
  Layers,
  ChevronRight,
} from 'lucide-react';
import { AwsS3Icon, AzureBlobIcon, GoogleCloudIcon } from './CloudProviderIcons';
import { BLOB_STORE_TYPE_IDS } from './blobStoreFormMachine';

import './BlobStoreTypeSelector.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

export type BlobStoreTypeId =
  | typeof BLOB_STORE_TYPE_IDS.FILE
  | typeof BLOB_STORE_TYPE_IDS.S3
  | typeof BLOB_STORE_TYPE_IDS.AZURE
  | typeof BLOB_STORE_TYPE_IDS.GOOGLE
  | typeof BLOB_STORE_TYPE_IDS.GROUP;

export interface BlobStoreTypeOption {
  id: BlobStoreTypeId;
  label: string;
  description: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
  accentClass: string;
}

const BLOB_STORE_OPTIONS: BlobStoreTypeOption[] = [
  {
    id: BLOB_STORE_TYPE_IDS.FILE,
    label: BlobStoresStrings.TYPE_SELECTOR.FILE,
    description: BlobStoresStrings.TYPE_SELECTOR.LOCAL_FILESYSTEM_STORAGE_SIMPLE_FAST,
    icon: HardDrive,
    accentClass: 'blob-type-selector__card--file',
  },
  {
    id: BLOB_STORE_TYPE_IDS.S3,
    label: BlobStoresStrings.TYPE_SELECTOR.AMAZON_S3,
    description: BlobStoresStrings.TYPE_SELECTOR.AWS_OBJECT_STORAGE_SCALABLE_DURABLE,
    icon: AwsS3Icon,
    accentClass: 'blob-type-selector__card--s3',
  },
  {
    id: BLOB_STORE_TYPE_IDS.AZURE,
    label: BlobStoresStrings.TYPE_SELECTOR.AZURE_BLOB,
    description: BlobStoresStrings.TYPE_SELECTOR.MICROSOFT_AZURE_BLOB_STORAGE_ENTERPRISE,
    icon: AzureBlobIcon,
    accentClass: 'blob-type-selector__card--azure',
  },
  {
    id: BLOB_STORE_TYPE_IDS.GOOGLE,
    label: BlobStoresStrings.TYPE_SELECTOR.GOOGLE_CLOUD,
    description: BlobStoresStrings.TYPE_SELECTOR.GOOGLE_CLOUD_STORAGE_OBJECT_STORAGE,
    icon: GoogleCloudIcon,
    accentClass: 'blob-type-selector__card--google',
  },
  {
    id: BLOB_STORE_TYPE_IDS.GROUP,
    label: BlobStoresStrings.TYPE_SELECTOR.GROUP,
    description: BlobStoresStrings.TYPE_SELECTOR.AGGREGATE_MULTIPLE_BLOB_STORES_SUPPORTS,
    icon: Layers,
    accentClass: 'blob-type-selector__card--group',
  },
];

export interface BlobStoreTypeSelectorProps {
  selectedType: BlobStoreTypeId | null;
  onSelect: (type: BlobStoreTypeId) => void;
  disabled?: boolean;
}

export function BlobStoreTypeSelector({
  selectedType,
  onSelect,
  disabled = false,
}: BlobStoreTypeSelectorProps) {
  return (
    <Box className="blob-type-selector">
      <Heading size="4" mb="2" weight="medium">
        {BlobStoresStrings.TYPE_SELECTOR.CHOOSE_STORAGE_TYPE}
      </Heading>
      <Text size="2" color="gray" mb="6" as="p">
        {BlobStoresStrings.TYPE_SELECTOR.SELECT_BLOB_STORE_TYPE_MATCHES}
      </Text>

      <Box className="blob-type-selector__grid" role="listbox" aria-label={BlobStoresStrings.TYPE_SELECTOR.BLOB_STORE_TYPES_ARIA}>
        {BLOB_STORE_OPTIONS.map((opt) => {
          const Icon = opt.icon;
          const isSelected = selectedType === opt.id;
          return (
            <button
              type="button"
              key={opt.id}
              role="option"
              aria-selected={isSelected}
              className={`blob-type-selector__card ${opt.accentClass} ${
                isSelected ? 'blob-type-selector__card--selected' : ''
              }`}
              onClick={() => !disabled && onSelect(opt.id)}
              disabled={disabled}
            >
              <Flex align="start" gap="4">
                <Box className="blob-type-selector__icon-wrap">
                  <Icon size={45} className="blob-type-selector__icon" />
                </Box>
                <Box style={{ flex: 1, minWidth: 0 }}>
                  <Text weight="bold" size="3" mb="1" as="p">
                    {opt.label}
                  </Text>
                  <Text size="2" color="gray" style={{ lineHeight: 1.5 }} as="p">
                    {opt.description}
                  </Text>
                </Box>
                <ChevronRight
                  size={20}
                  className={`blob-type-selector__chevron ${
                    isSelected ? 'blob-type-selector__chevron--visible' : ''
                  }`}
                />
              </Flex>
            </button>
          );
        })}
      </Box>
    </Box>
  );
}

export default BlobStoreTypeSelector;
