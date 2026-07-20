
import {APIConstants, ExtJS} from '@globalart/astrum-ui-plugin';

const {
  REST: {
    PUBLIC: {SSL_CERTIFICATES: sslCertificatesUrl}
  }
} = APIConstants;

const singleSslCertificatesUrl = (id) => `${sslCertificatesUrl}/${encodeURIComponent(id)}`;

const createSslCertificatesUrl = sslCertificatesUrl;

export const remoteHostRequestData = (value) => {
    const hasProtocol = value.startsWith('http');
  
    const urlStr = hasProtocol ? value : 'https://' + value;
  
    const {protocol, hostname, port} = new URL(urlStr);
  
    const portNumber = parseInt(port) || null;
    const protocolHint = hasProtocol ? protocol : null;
  
    return [hostname, portNumber, protocolHint];
  };

export const URLS = {
  sslCertificatesUrl,
  singleSslCertificatesUrl,
  createSslCertificatesUrl
};

export const canDeleteCertificate = () => ExtJS.checkPermission('astrum:ssl-truststore:delete');
export const canCreateCertificate = () => ExtJS.checkPermission('astrum:ssl-truststore:create');
