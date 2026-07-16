
import React, { useState, useEffect } from 'react';
import SwaggerUI from "swagger-ui-react"
import {
  ContentBody,
  ExtJS,
  Page,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {
  NxTextInput,
  NxFormGroup,
  NxButton
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import 'swagger-ui-react/swagger-ui.css';
import './Api.scss';

function requestInterceptor(request) {
  request.headers['NX-ANTI-CSRF-TOKEN'] = (document.cookie.match('(^|; )NX-ANTI-CSRF-TOKEN=([^;]*)')||0)[2];
  request.headers['X-Astrum-UI'] = true;
  return request;
}

function responseInterceptor(response) {
  let data = response.data;

  if (typeof response.data === 'string') {
    try {
      data = JSON.parse(response.data);
    } catch (e) {
      console.error(e);
    }
  }

  if(data.tags) {
    const tags = data.tags.sort((a, b) => a.name.localeCompare(b.name));

    response.body.tags = tags;
    response.data = {...data, tags};
    
    try {
      const text = JSON.parse(response.text);
      response.text = JSON.stringify({...text, tags});
    } catch (e) {
      console.error(e);
    }
  }

  return response;
}

export default function Api() {
  const [searchTerm, setSearchTerm] = useState('');

  // Comprehensive search functionality
  useEffect(() => {
    const filterOperations = () => {
      const swaggerContainer = document.querySelector('.swagger-ui');
      if (!swaggerContainer) {
        return;
      }

      const operations = swaggerContainer.querySelectorAll('.opblock');
      const lowerSearchTerm = searchTerm.toLowerCase();

      operations.forEach(operation => {
        const operationText = operation.textContent.toLowerCase();
        const shouldShow = !lowerSearchTerm || operationText.includes(lowerSearchTerm);
        
        if (shouldShow) {
          operation.classList.remove('hidden');
        } else {
          operation.classList.add('hidden');
        }
      });
    };

    // Add delay to allow Swagger UI to render
    const timeoutId = setTimeout(filterOperations, 100);
    return () => clearTimeout(timeoutId);
  }, [searchTerm]);

  return (
    <Page>
      <PageHeader>
        <PageTitle
            icon={UIStrings.API.MENU.icon}
            text={UIStrings.API.MENU.text}
            description={UIStrings.API.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-api">
        <div className="api-search-container">
          <NxFormGroup 
            label="Search Operations" 
            sublabel="Search by operation name, path, or description"
          >
            <NxTextInput
              placeholder="Enter search term..."
              value={searchTerm}
              onChange={setSearchTerm}
              data-analytics-id="swagger-ui-search-input"
            />
          </NxFormGroup>
          {searchTerm && (
            <div className="api-search-feedback">
              <strong>Filtering:</strong> Operations containing "{searchTerm}"
              <NxButton
                variant="tertiary"
                className="api-search-clear-btn"
                onClick={() => setSearchTerm('')}
                data-analytics-id="swagger-ui-search-clear-btn"
              >
                Clear
              </NxButton>
            </div>
          )}
        </div>
        <SwaggerUI
          url={ExtJS.urlOf('/service/rest/swagger.json')}
          requestInterceptor={requestInterceptor}
          responseInterceptor={responseInterceptor}
          defaultModelsExpandDepth={-1}
        />
      </ContentBody>
    </Page>
  );
}
