
/*global Ext, NX*/

/**
 * Search criteria mixin.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.search.SearchCriteria', {

  searchCriteria: true,

  filter: function () {
    var me = this;
    if (me.value) {
      return { property: me.criteriaId, value: me.value };
    }
    return undefined;
  }

});
