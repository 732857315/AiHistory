export function start() {
    wasm.start();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_function_1f9d30630b8b1d3d: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_undefined_8865fb403f8fe9d8: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_2e0e7dee9f701a71: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_rethrow_cb2e88c6b2a16733: function(arg0) {
            throw takeObject(arg0);
        },
        __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_dcc1a90847f04c41: function(arg0) {
            getObject(arg0)._wbg_cb_unref();
        },
        __wbg_addColorStop_2fe19c9df420d0a6: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addColorStop(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_addEventListener_4d0db17c671ea324: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments); },
        __wbg_altKey_cfb4c4fab9a7d0d0: function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        },
        __wbg_arc_03f7717b2e38682a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_beginPath_8598d895c13f1c86: function(arg0) {
            getObject(arg0).beginPath();
        },
        __wbg_bezierCurveTo_4e6afc9957d235e6: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).bezierCurveTo(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_cancel_fa9c526b1bfea1fa: function(arg0) {
            getObject(arg0).cancel();
        },
        __wbg_canvas_2b619d4767b78c03: function(arg0) {
            const ret = getObject(arg0).canvas;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_clientWidth_08d5512595aacb73: function(arg0) {
            const ret = getObject(arg0).clientWidth;
            return ret;
        },
        __wbg_clip_275e70d2dc182fec: function(arg0) {
            getObject(arg0).clip();
        },
        __wbg_closePath_355eb6da179dd9c2: function(arg0) {
            getObject(arg0).closePath();
        },
        __wbg_close_d6be50a702eae8fc: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).close();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_closest_6abd1bf5d7559cda: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).closest(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_connect_30bfff6aec59fc83: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).connect(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createElement_74049073a11f9c31: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createGain_b6551e1ad1609779: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createGain();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createLinearGradient_d67a7d18d11c0b62: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createLinearGradient(arg1, arg2, arg3, arg4);
            return addHeapObject(ret);
        },
        __wbg_createOscillator_7a285541e405b9ae: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).createOscillator();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createRadialGradient_e54676bfd0a6ae40: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            const ret = getObject(arg0).createRadialGradient(arg1, arg2, arg3, arg4, arg5, arg6);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_ctrlKey_8b5101745e8782fa: function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        },
        __wbg_currentTime_eb1592316a1d4450: function(arg0) {
            const ret = getObject(arg0).currentTime;
            return ret;
        },
        __wbg_defaultPrevented_5af41e1dd07ed819: function(arg0) {
            const ret = getObject(arg0).defaultPrevented;
            return ret;
        },
        __wbg_destination_05471d87031cba86: function(arg0) {
            const ret = getObject(arg0).destination;
            return addHeapObject(ret);
        },
        __wbg_disconnect_513def74b1e0d5cc: function() { return handleError(function (arg0) {
            getObject(arg0).disconnect();
        }, arguments); },
        __wbg_documentElement_3f718a9a39a893d3: function(arg0) {
            const ret = getObject(arg0).documentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_document_9854e03c05fc8834: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_drawImage_93d4867540340235: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).drawImage(getObject(arg1), arg2, arg3);
        }, arguments); },
        __wbg_exitFullscreen_bed3a92784bda110: function(arg0) {
            getObject(arg0).exitFullscreen();
        },
        __wbg_exponentialRampToValueAtTime_f53d677fb4544dd9: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).exponentialRampToValueAtTime(arg1, arg2);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_fillRect_0ef59adb9acb7d06: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_1bcec8b81ad73bd0: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fillText_b84e10d5843dc028: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).fillText(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5);
        }, arguments); },
        __wbg_fill_fc5e02a06cc26e92: function(arg0) {
            getObject(arg0).fill();
        },
        __wbg_frequency_0f39b5e36f7073c6: function(arg0) {
            const ret = getObject(arg0).frequency;
            return addHeapObject(ret);
        },
        __wbg_fullscreenElement_3f3cceda5e10c3b1: function(arg0) {
            const ret = getObject(arg0).fullscreenElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_gain_29777386775059c0: function(arg0) {
            const ret = getObject(arg0).gain;
            return addHeapObject(ret);
        },
        __wbg_getAttribute_061ad00c16e2f622: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getContext_635e36719cad2623: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getElementById_cc94972b404e4eaa: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_getItem_4c80b7663197c7ff: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getVoices_a94654c567778a75: function(arg0) {
            const ret = getObject(arg0).getVoices();
            return addHeapObject(ret);
        },
        __wbg_get_unchecked_288889d017702237: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_globalAlpha_9ac73c8e4c5a15ad: function(arg0) {
            const ret = getObject(arg0).globalAlpha;
            return ret;
        },
        __wbg_hasAttribute_b009da6c546736e9: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).hasAttribute(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_hash_c4083658cccb641b: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_height_fc97e1a0c2e7331f: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_hidden_2ba371ecafe013fa: function(arg0) {
            const ret = getObject(arg0).hidden;
            return ret;
        },
        __wbg_id_957842593c7d0e05: function(arg0, arg1) {
            const ret = getObject(arg1).id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_innerHTML_62f0c5c2a76942db: function(arg0, arg1) {
            const ret = getObject(arg1).innerHTML;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_innerWidth_1bc526614080537c: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).innerWidth;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_instanceof_CanvasRenderingContext2d_769208c72dcbf5e6: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Element_818e11074cdb63b5: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_0a30c11fbbf41841: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_5c33d1de59c09c49: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlSelectElement_a90d012104ce2fb3: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLSelectElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_KeyboardEvent_0176f04c0ee63f6d: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof KeyboardEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_SpeechSynthesisVoice_eca98c9c1f7f7678: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof SpeechSynthesisVoice;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_82d71df4eddf88bc: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_key_1193871533b99ae5: function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_lang_124ac9807c8fd7c7: function(arg0, arg1) {
            const ret = getObject(arg1).lang;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_language_2efc316af90b406f: function(arg0, arg1) {
            const ret = getObject(arg1).language;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_d4bdea10311bd9cf: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_lineTo_63fac7d60279d95e: function(arg0, arg1, arg2) {
            getObject(arg0).lineTo(arg1, arg2);
        },
        __wbg_linearRampToValueAtTime_f87a0c73867277b7: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).linearRampToValueAtTime(arg1, arg2);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_localStorage_567c24950edbc178: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_location_9c46e8ada95d3174: function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        },
        __wbg_matchMedia_8a4857f947f11f82: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_matches_778d0873b064b1bf: function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        },
        __wbg_metaKey_fb78da4d237d5bbb: function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        },
        __wbg_moveTo_037e3deefec91ae1: function(arg0, arg1, arg2) {
            getObject(arg0).moveTo(arg1, arg2);
        },
        __wbg_navigator_2156486643462a87: function(arg0) {
            const ret = getObject(arg0).navigator;
            return addHeapObject(ret);
        },
        __wbg_new_afd97fa36645d359: function() { return handleError(function () {
            const ret = new lAudioContext();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_with_text_de120aa95bd08db1: function() { return handleError(function (arg0, arg1) {
            const ret = new SpeechSynthesisUtterance(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_offsetLeft_eef008f2ccb3b242: function(arg0) {
            const ret = getObject(arg0).offsetLeft;
            return ret;
        },
        __wbg_offsetWidth_1d736667aa04ab3a: function(arg0) {
            const ret = getObject(arg0).offsetWidth;
            return ret;
        },
        __wbg_pending_decfa19ee64ef69d: function(arg0) {
            const ret = getObject(arg0).pending;
            return ret;
        },
        __wbg_preventDefault_af59afb0f0a02e20: function(arg0) {
            getObject(arg0).preventDefault();
        },
        __wbg_quadraticCurveTo_4f57843d21c4f90d: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).quadraticCurveTo(arg1, arg2, arg3, arg4);
        },
        __wbg_queueMicrotask_9833f9a49df95a49: function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        },
        __wbg_queueMicrotask_a72f977e97f23c5f: function(arg0) {
            queueMicrotask(getObject(arg0));
        },
        __wbg_removeAttribute_2f2700a6f933a6be: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_repeat_8e70ccf8a9875d04: function(arg0) {
            const ret = getObject(arg0).repeat;
            return ret;
        },
        __wbg_requestAnimationFrame_7b526ab7aa550c74: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments); },
        __wbg_requestFullscreen_4eb8242c77c753fd: function() { return handleError(function (arg0) {
            getObject(arg0).requestFullscreen();
        }, arguments); },
        __wbg_resolve_0076e10020304ede: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_restore_c93ba7571816b182: function(arg0) {
            getObject(arg0).restore();
        },
        __wbg_resume_262977d0e323b264: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).resume();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_rotate_5273f09fed0af920: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).rotate(arg1);
        }, arguments); },
        __wbg_save_f32554f1747071d1: function(arg0) {
            getObject(arg0).save();
        },
        __wbg_scale_a6ef716accd967d5: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).scale(arg1, arg2);
        }, arguments); },
        __wbg_scrollIntoView_f6c27be4871460ef: function(arg0) {
            getObject(arg0).scrollIntoView();
        },
        __wbg_setAttribute_9e7d603908f63705: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setItem_c5fb0966b484ecd3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setTargetAtTime_5edbac64de300ef1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).setTargetAtTime(arg1, arg2, arg3);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_setValueAtTime_2edc098944a654d2: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setValueAtTime(arg1, arg2);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_set_fillStyle_1a796925e818ac7c: function(arg0, arg1) {
            getObject(arg0).fillStyle = getObject(arg1);
        },
        __wbg_set_fillStyle_27f88552f4d2ab31: function(arg0, arg1) {
            getObject(arg0).fillStyle = getObject(arg1);
        },
        __wbg_set_fillStyle_a2961b4d44e572af: function(arg0, arg1, arg2) {
            getObject(arg0).fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_1f60a05a2544a2ff: function(arg0, arg1, arg2) {
            getObject(arg0).font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_globalAlpha_ae4b85201dda64c5: function(arg0, arg1) {
            getObject(arg0).globalAlpha = arg1;
        },
        __wbg_set_globalCompositeOperation_8d071d7528280f24: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).globalCompositeOperation = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_height_c9789c1c77eaedff: function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        },
        __wbg_set_innerHTML_7af59a832a09a074: function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lang_fdb29fe7a7f6b538: function(arg0, arg1, arg2) {
            getObject(arg0).lang = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineCap_c9b7bdd3b7f5ce74: function(arg0, arg1, arg2) {
            getObject(arg0).lineCap = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineJoin_c1780f2096e6f970: function(arg0, arg1, arg2) {
            getObject(arg0).lineJoin = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_cc15473552c60c9c: function(arg0, arg1) {
            getObject(arg0).lineWidth = arg1;
        },
        __wbg_set_rate_caba1c2a5e041669: function(arg0, arg1) {
            getObject(arg0).rate = arg1;
        },
        __wbg_set_scrollLeft_02099c8a59d32652: function(arg0, arg1) {
            getObject(arg0).scrollLeft = arg1;
        },
        __wbg_set_shadowBlur_216915d92b179fbe: function(arg0, arg1) {
            getObject(arg0).shadowBlur = arg1;
        },
        __wbg_set_shadowColor_0e2bd5d3400b0d3b: function(arg0, arg1, arg2) {
            getObject(arg0).shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_shadowOffsetX_0dbcf4ca8295e0c3: function(arg0, arg1) {
            getObject(arg0).shadowOffsetX = arg1;
        },
        __wbg_set_shadowOffsetY_de495fcc5a5ae809: function(arg0, arg1) {
            getObject(arg0).shadowOffsetY = arg1;
        },
        __wbg_set_strokeStyle_a36f39be93a6cebc: function(arg0, arg1) {
            getObject(arg0).strokeStyle = getObject(arg1);
        },
        __wbg_set_strokeStyle_d51608fa918b53d4: function(arg0, arg1, arg2) {
            getObject(arg0).strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_1bda1733d57574ed: function(arg0, arg1, arg2) {
            getObject(arg0).textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_d5ba548751584f49: function(arg0, arg1, arg2) {
            getObject(arg0).textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textContent_6d6fc559f198055f: function(arg0, arg1, arg2) {
            getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_title_26e2c1657f5010ca: function(arg0, arg1, arg2) {
            getObject(arg0).title = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_59c4e91d57bb96dc: function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_OscillatorType[arg1];
        },
        __wbg_set_value_bf45a2dfcba57147: function(arg0, arg1) {
            getObject(arg0).value = arg1;
        },
        __wbg_set_value_f6901ece9a8a1a2a: function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_value_fbd659f94bfb9181: function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_voice_3e1a3ef98a8d5ecd: function(arg0, arg1) {
            getObject(arg0).voice = getObject(arg1);
        },
        __wbg_set_volume_d84050ffb0189f47: function(arg0, arg1) {
            getObject(arg0).volume = arg1;
        },
        __wbg_set_width_b0e1267db4b196b5: function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        },
        __wbg_speak_0522b6116a03fd31: function(arg0, arg1) {
            getObject(arg0).speak(getObject(arg1));
        },
        __wbg_speaking_bbf159df39686677: function(arg0) {
            const ret = getObject(arg0).speaking;
            return ret;
        },
        __wbg_speechSynthesis_9178be485e9e2ef1: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).speechSynthesis;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_start_3f09c6b8021c0242: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).start(arg1);
        }, arguments); },
        __wbg_state_54551b9345fb007f: function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_AudioContextState.indexOf(ret) + 1 || 4) - 1;
        },
        __wbg_static_accessor_GLOBAL_266715b9d96ba635: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_10fb7dc1ae063179: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_0b583911f537483a: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_d7f903d1508cbdc4: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_stop_3744a2b843370b53: function() { return handleError(function (arg0) {
            getObject(arg0).stop();
        }, arguments); },
        __wbg_stop_ff597eb931e2e4f9: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).stop(arg1);
        }, arguments); },
        __wbg_stroke_385d731098398489: function(arg0) {
            getObject(arg0).stroke();
        },
        __wbg_suspend_2fbcec238306d5b7: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).suspend();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_tagName_e83500df63beeee1: function(arg0, arg1) {
            const ret = getObject(arg1).tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_target_38ae9feb025b820c: function(arg0) {
            const ret = getObject(arg0).target;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_textContent_acd42c63564a445e: function(arg0, arg1) {
            const ret = getObject(arg1).textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_then_c949d5a25a4e78f8: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_then_e71170d78fcf8954: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_translate_b75b7d842d89a889: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).translate(arg1, arg2);
        }, arguments); },
        __wbg_value_05305a761dfa3e0e: function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_value_e2b1e9c07e7e5815: function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_width_3d0dce3d9892e35e: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 56, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1045);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [F64], shim_idx: 10, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_310);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 9, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_309);
            return addHeapObject(ret);
        },
        __wbindgen_generic_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./aihistory_bg.js": import0,
    };
}

const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
function __wasm_bindgen_func_elem_309(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_309(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1045(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_1045(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_310(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_310(arg0, arg1, arg2);
}


const __wbindgen_enum_AudioContextState = ["suspended", "running", "closed"];


const __wbindgen_enum_OscillatorType = ["sine", "square", "sawtooth", "triangle", "custom"];

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_export4(state.a, state.b));

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_export4(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('aihistory_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
