declare module 'swagger-ui-dist/swagger-ui-es-bundle.js' {
  interface SwaggerRequest {
    url: string;
    method?: string;
    headers: Record<string, string>;
  }

  interface SwaggerOptions {
    domNode: HTMLElement;
    url: string;
    deepLinking?: boolean;
    docExpansion?: 'list' | 'full' | 'none';
    defaultModelsExpandDepth?: number;
    displayRequestDuration?: boolean;
    filter?: boolean;
    tryItOutEnabled?: boolean;
    validatorUrl?: string | null;
    requestInterceptor?: (request: SwaggerRequest) => SwaggerRequest;
  }

  export default function SwaggerUI(options: SwaggerOptions): unknown;
}
