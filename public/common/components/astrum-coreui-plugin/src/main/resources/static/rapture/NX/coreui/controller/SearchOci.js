
/*global Ext, NX*/

/**
 * OCI repository search contribution.
 *
 * @since 3.91
 */
Ext.define('NX.coreui.controller.SearchOci', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function () {
    var me = this,
        search = me.getController('NX.coreui.controller.Search');

    // OCI uses its own attribute namespace for proper format separation
    search.registerCriteria([
      {
        id: 'attributes.oci.imageName',
        group: NX.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: NX.I18n.get('SearchOci_Image_Name_FieldLabel'),
          width: 300
        }
      },
      {
        id: 'attributes.oci.imageTag',
        group: NX.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: NX.I18n.get('SearchOci_Image_Tag_FieldLabel')
        }
      },
      {
        id: 'assets.attributes.oci.content_digest',
        group: NX.I18n.get('SearchOci_Group'),
        config: {
          format: 'oci',
          fieldLabel: NX.I18n.get('SearchOci_ContentDigest_FieldLabel'),
          width: 500
        }
      }
    ], me);

    search.registerFilter({
      id: 'oci',
      name: 'OCI',
      text: NX.I18n.get('SearchOci_Text'),
      description: NX.I18n.get('SearchOci_Description'),
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
