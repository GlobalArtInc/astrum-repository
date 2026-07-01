
/*global Ext, NX*/

/**
 * HuggingFace repository search contribution.
 *
 * @since 3.19
 */
Ext.define('NX.coreui.controller.SearchHuggingFace', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    const me = this,
        search = me.getController('NX.coreui.controller.Search');

    search.registerFilter({
      id: 'huggingface',
      name: 'huggingFace',
      text: NX.I18n.get('SearchHuggingFace_Text'),
      description: NX.I18n.get('SearchHuggingFace_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'huggingface', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
