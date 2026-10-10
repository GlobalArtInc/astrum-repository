
/*global Ext, AS*/

/**
 * HuggingFace repository search contribution.
 *
 * @since 3.19
 */
Ext.define('AS.coreui.controller.SearchHuggingFace', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    const me = this,
        search = me.getController('AS.coreui.controller.Search');

    search.registerFilter({
      id: 'huggingface',
      name: 'huggingFace',
      text: AS.I18n.get('SearchHuggingFace_Text'),
      description: AS.I18n.get('SearchHuggingFace_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'huggingface', hidden: true},
        {id: 'name.raw'},
        {id: 'version'}
      ]
    }, me);
  }
});
