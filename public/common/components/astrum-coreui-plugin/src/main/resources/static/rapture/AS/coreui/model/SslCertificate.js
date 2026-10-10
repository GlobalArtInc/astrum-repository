
/*global Ext, AS*/

/**
 * Ssl Certificate model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.SslCertificate', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string'},
    {name: 'fingerprint', type: 'string'},
    {name: 'pem', type: 'string'},
    {name: 'serialNumber', type: 'string'},
    {name: 'subjectCommonName', type: 'string'},
    {name: 'subjectOrganization', type: 'string'},
    {name: 'subjectOrganizationalUnit', type: 'string'},
    {name: 'issuerCommonName', type: 'string'},
    {name: 'issuerOrganization', type: 'string'},
    {name: 'issuerOrganizationalUnit', type: 'string'},
    {name: 'issuedOn', type: 'int'},
    {name: 'expiresOn', type: 'int'},
    {name: 'inTrustStore', type: 'boolean'}
  ]
});
