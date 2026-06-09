
/*global Ext, NX*/

/**
 * **{@link Ext.form.action.DirectSubmit}** overrides (see inline comments marked with &lt;override/&gt;
 *
 * See: https://support.sencha.com/index.php#ticket-16118
 *
 * See: https://support.sencha.com/index.php#ticket-16102
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.action.DirectSubmit', {
  override: 'Ext.form.action.DirectSubmit',

  submitEmptyText: false,

  doSubmit: function () {
    var me = this,
        form = me.form,
        api = form.api,
        fn = api.submit,
        callback = Ext.Function.bind(me.onComplete, me),
        formInfo = me.buildForm(),
        options, formEl;

    if (!Ext.isFunction(fn)) {
      //<debug>
      var fnName = fn;
      //</debug>

      api.update = fn = Ext.direct.Manager.parseMethod(fn);
      //<override> avoid cleanup as that resets values of file upload fields
      //me.cleanup(formInfo);
      //</override>

      //<debug>
      if (!Ext.isFunction(fn)) {
        Ext.Error.raise('Cannot resolve Ext.Direct API method ' + fnName);
      }
      //</debug>
    }

    if (me.timeout || form.timeout) {
      options = {
        timeout: me.timeout * 1000 || form.timeout * 1000
      };
    }

    //<override> call using field values if direct function formHandler = false
    //fn.call(NX.global, formInfo.formEl, callback, me, options);
    if (fn.directCfg.method.formHandler) {
      formEl = formInfo.formEl;
    }
    else {
      formEl = me.getParams(true);
      Ext.Object.each(formEl, function (key, value) {
        if (Ext.typeOf(value) === 'date') {
          formEl[key] = Ext.Date.format(value, 'Y-m-d\\TH:i:s.uP');
        }
      });
    }
    fn.call(NX.global, formEl, callback, me, options);
    //</override>
    me.cleanup(formInfo);
  }

});
