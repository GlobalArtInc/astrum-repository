
/*global Ext*/

/**
 * Validation helpers.
 *
 * @since 3.0
 */
Ext.define('AS.util.Validator', {
  singleton: true,
  requires: [
      'Ext.form.field.VTypes',
      'AS.I18n'
  ],

  /**
   * Changes to this property should be sync'd in:
   * components/astrum-validation/src/main/java/org/globalart/astrum/validation/constraint/NamePatternConstants.java
   * @private
   */
  asNameRegex : /^[a-zA-Z0-9\-]{1}[a-zA-Z0-9_\-\.]*$/,

  /**
   * Removes the constraint for a maximum of 6 characters in the last element of the domain name, otherwise
   * is the same as default ExtJS email vtype.
   * @private
   */
  asEmailRegex : /^(")?(?:[^\."])(?:(?:[\.])?(?:[\w\-!#$%&'*+/=?^_`{|}~]))*\1@(\w[\-\w]*\.?){1,5}([A-Za-z]){2,60}$/,

  /**
   * A regular expression to detect a valid hostname according to RFC 1123.
   * See also http-headers-patterns.properties and HostnameValidator.java for other uses of this regex.
   * @private
   */
  asRfc1123HostRegex: new RegExp(
    "^(((([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]))|" +
     "(\\[(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\\])|" +
     "(\\[((?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4})*)?)::((?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4})*)?)\\])|" +
     "(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\\-]*[a-zA-Z0-9])\\.)*([A-Za-z0-9]|" +
     "[A-Za-z0-9][A-Za-z0-9\\-]*[A-Za-z0-9]))(:([0-9]+))?$"
  ),

  /**
   * A regular expression to detect a possibly valid URL
   * @private
   */
  asUrlRegex: /^https?:\/\/[^"<>^`{|}]+$/i,

  /**
   * Looser URL regex for OCI proxy remote URLs. Accepts {@code http://}, {@code https://}, and
   * the OCI URI conventions {@code oci://} and {@code oci+http://}. The backend's
   * {@code OciProxyRepositoryApiRequestToConfigurationConverter#normalizeRemoteUrl} normalises
   * those OCI schemes to {@code https}/{@code http} on save, so the UI must accept what the
   * server already accepts.
   * @private
   */
  asOciUrlRegex: /^(https?|oci(\+http)?):\/\/[^"<>^`{|}]+$/i,

  /**
   * A regular expression to detect whether we have leading and trailing white space
   *
   * @private
   */
  asLeadingAndTrailingWhiteSpaceRegex : /^[ \s]+|[ \s]+$/,

  /**
   * Regular expression to validate docker subdomain
   * @private
   */
  asSubdomainRegex : /^[a-zA-Z](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$/,

  /**
   * @public
   * @param vtype {object}
   */
  registerVtype: function(vtype) {
    Ext.apply(Ext.form.field.VTypes, vtype);
  },

  constructor: function () {
    var me = this;

    me.vtypes = [
      {
        'as-name': function(val) {
          return AS.util.Validator.asNameRegex.test(val);
        },
        'as-nameText': AS.I18n.get('Util_Validator_Text'),
        'as-email': function(val) {
          return AS.util.Validator.asEmailRegex.test(val);
        },
        'as-emailText': Ext.form.field.VTypes.emailText,
        'as-hostname': function(val) {
          return AS.util.Validator.asRfc1123HostRegex.test(val);
        },
        'as-hostnameText': AS.I18n.get('Util_Validator_Hostname'),
        'as-trim': function(val) {
          return !AS.util.Validator.asLeadingAndTrailingWhiteSpaceRegex.test(val);
        },
        'as-trimText': AS.I18n.get('Util_Validator_Trim'),
        'as-url': function(val) {
          return AS.util.Validator.asUrlRegex.test(val);
        },
        'as-urlText': AS.I18n.get('Util_Validator_Url'),
        'as-oci-url': function(val) {
          return AS.util.Validator.asOciUrlRegex.test(val);
        },
        'as-oci-urlText': AS.I18n.get('Util_Validator_OciUrl'),
        'as-subdomain': function(val) {
          return AS.util.Validator.asSubdomainRegex.test(val);
        },
        'as-subdomainText': AS.I18n.get('Util_Validator_Subdomain_Text'),
      }
    ];

    Ext.each(me.vtypes, function(vtype) {
      me.registerVtype(vtype);
    });
  }

});
