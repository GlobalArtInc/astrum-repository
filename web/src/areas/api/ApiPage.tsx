import 'swagger-ui-dist/swagger-ui.css';
import { useEffect, useRef } from 'react';
import SwaggerUI from 'swagger-ui-dist/swagger-ui-es-bundle.js';
import { useT } from '../../i18n';
import { Page } from '../../ui';
import styles from './Api.module.css';
import { DOCUMENT, withSession } from './model';

export default function ApiPage() {
  const t = useT();
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const node = host.current;
    if (!node) {
      return;
    }
    SwaggerUI({
      domNode: node,
      url: DOCUMENT,
      deepLinking: true,
      docExpansion: 'list',
      defaultModelsExpandDepth: 0,
      displayRequestDuration: true,
      filter: true,
      validatorUrl: null,
      requestInterceptor: withSession,
    });
    return () => {
      node.replaceChildren();
    };
  }, []);
  return (
    <Page
      gutter="small"
      title={t('api.page.title')}
      description={t('api.page.description')}
      breadcrumbs={[{ label: t('api.crumbs.settings'), to: '/settings' }, { label: t('api.page.title') }]}
    >
      <div ref={host} className={styles.swagger} aria-label={t('api.page.title')} role="region" />
    </Page>
  );
}
