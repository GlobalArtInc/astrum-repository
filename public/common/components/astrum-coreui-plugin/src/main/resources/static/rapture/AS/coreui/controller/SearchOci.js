
/*global Ext, AS*/

/**
 * OCI repository search contribution.
 *
 * @since 3.91
 */
Ext.define('AS.coreui.controller.SearchOci', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this,
        search = me.getController('AS.coreui.controller.Search');

    // OCI uses its own attribute namespace for proper format separation
    search.registerCriteria([
      {
        id: 'attributes.oci.imageName',
        group: AS.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: AS.I18n.get('SearchOci_Image_Name_FieldLabel'),
          width: 300
        }
      },
      {
        id: 'attributes.oci.imageTag',
        group: AS.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: AS.I18n.get('SearchOci_Image_Tag_FieldLabel')
        }
      },
      {
        id: 'assets.attributes.oci.content_digest',
        group: AS.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: AS.I18n.get('SearchOci_ContentDigest_FieldLabel'),
          width: 500
        }
      }
    ], me);

    search.registerFilter({
      id: 'oci',
      name: 'OCI',
      text: AS.I18n.get('SearchOci_Text'),
      description: AS.I18n.get('SearchOci_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'oci', hidden: true},
        {id: 'attributes.oci.imageName'},
        {id: 'attributes.oci.imageTag'},
        {id: 'assets.attributes.oci.content_digest'}
      ]
    }, me);
  }

});
