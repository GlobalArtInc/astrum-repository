

import React, { useMemo } from 'react';
import { Box, Text, Flex } from '@radix-ui/themes';
import { Eye } from 'lucide-react';
import { interpretExpression } from './cselValidator';

import './ExpressionPreview.scss';

interface ExpressionPreviewProps {
  expression: string;
}

/**
 * ExpressionPreview - Shows a human-readable interpretation of the CSEL expression
 *
 * This is a read-only, best-effort preview that helps users understand what
 * their expression will match. It does NOT block save if parsing fails.
 */
export function ExpressionPreview({ expression }: ExpressionPreviewProps) {
  const interpretation = useMemo(() => interpretExpression(expression), [expression]);

  if (!(expression.trim() && interpretation.success)) {
    return null;
  }

  return (
    <Box className="expression-preview">
      <Flex align="center" gap="2" className="expression-preview__header">
        <Eye size={14} />
        <Text size="1" weight="medium">Expression Preview</Text>
      </Flex>
      <Text as="p" size="2" className="expression-preview__text">
        {interpretation.text}
      </Text>
    </Box>
  );
}

export default ExpressionPreview;


