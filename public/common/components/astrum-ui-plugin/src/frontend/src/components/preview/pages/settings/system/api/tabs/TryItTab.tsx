

import React, { useMemo } from 'react';
import { Box, Text } from '@radix-ui/themes';
import SwaggerUI from 'swagger-ui-react';

import { sliceSwaggerSpec, type MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import { swaggerRequestInterceptor, swaggerResponseInterceptor } from '../swaggerInterceptors';

import 'swagger-ui-react/swagger-ui.css';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';

export interface TryItTabProps {
  fullSwagger: Record<string, unknown> | null;
  row: MergedApiEndpoint;
  accessDenied: boolean;
}

export function TryItTab({ fullSwagger, row, accessDenied }: TryItTabProps) {
  const methodLower = row.httpMethod.toLowerCase() as 'get' | 'post' | 'put' | 'delete' | 'patch' | 'head' | 'options';

  const slim = useMemo(() => {
    if (!fullSwagger) {
      return null;
    }
    return sliceSwaggerSpec(fullSwagger, row.swaggerPathKey, methodLower);
  }, [fullSwagger, row.swaggerPathKey, methodLower]);

  if (!(fullSwagger && slim)) {
    return (
      <Text size="2" color="gray">
        {ApiStrings.TRY_IT_TAB.API_DOCUMENTATION_NOT_AVAILABLE_LOAD}
      </Text>
    );
  }

  return (
    <Box className="api-try-it-tab">
      {accessDenied && (
        <Box mb="3" p="2" className="api-try-it-tab__banner">
          <Text size="2">
            {ApiStrings.TRY_IT_TAB.YOU_MAY_NOT_HAVE_PERMISSION}
          </Text>
        </Box>
      )}
      <Box className="api-try-it-tab__swagger" data-testid="api-try-it-swagger">
        <SwaggerUI
          key={`${row.httpMethod}-${row.fullPath}`}
          spec={slim}
          requestInterceptor={swaggerRequestInterceptor}
          responseInterceptor={swaggerResponseInterceptor}
          defaultModelsExpandDepth={-1}
          docExpansion="full"
        />
      </Box>
    </Box>
  );
}
