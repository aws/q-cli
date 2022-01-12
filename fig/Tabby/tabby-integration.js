(function webpackUniversalModuleDefinition(root, factory) {
	if(typeof exports === 'object' && typeof module === 'object')
		module.exports = factory(require("@angular/core"), require("rxjs"), require("tabby-core"), require("tabby-terminal"));
	else if(typeof define === 'function' && define.amd)
		define(["@angular/core", "rxjs", "tabby-core", "tabby-terminal"], factory);
	else {
		var a = typeof exports === 'object' ? factory(require("@angular/core"), require("rxjs"), require("tabby-core"), require("tabby-terminal")) : factory(root["@angular/core"], root["rxjs"], root["tabby-core"], root["tabby-terminal"]);
		for(var i in a) (typeof exports === 'object' ? exports : root)[i] = a[i];
	}
})(global, function(__WEBPACK_EXTERNAL_MODULE__angular_core__, __WEBPACK_EXTERNAL_MODULE_rxjs__, __WEBPACK_EXTERNAL_MODULE_tabby_core__, __WEBPACK_EXTERNAL_MODULE_tabby_terminal__) {
return /******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./src/index.ts":
/*!**********************!*\
  !*** ./src/index.ts ***!
  \**********************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {


var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __metadata = (this && this.__metadata) || function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
const core_1 = __webpack_require__(/*! @angular/core */ "@angular/core");
const rxjs_1 = __webpack_require__(/*! rxjs */ "rxjs");
const cp = __webpack_require__(/*! child_process */ "child_process");
const tabby_core_1 = __importStar(__webpack_require__(/*! tabby-core */ "tabby-core"));
const tabby_terminal_1 = __importStar(__webpack_require__(/*! tabby-terminal */ "tabby-terminal"));
let runCommand = function (command, logger) {
    logger.info(command);
    cp.exec(command, (err) => {
        if (err && logger) {
            logger.error(err);
        }
    });
};
let FigModule = class FigModule {
    constructor(log, app) {
        this.activePane = null;
        this.index = 0;
        this.logger = log.create('fig');
        app.tabOpened$.subscribe(tab => {
            this.logger.info('New tab opened', tab.title);
            // this.mapping.set(tab, this.index)
            if (tab instanceof tabby_terminal_1.BaseTerminalTabComponent) {
                // this.attachTo(tab)
            }
            else if (tab instanceof tabby_core_1.SplitTabComponent) {
                tab.initialized$.subscribe(() => {
                    rxjs_1.merge(tab.tabAdded$, rxjs_1.from(tab.getAllTabs()) // tabAdded$ doesn't fire for tabs restored from saved sessions
                    ).subscribe(pane => {
                        if (pane instanceof tabby_terminal_1.BaseTerminalTabComponent) {
                            this.logger.info('New pane opened', pane.title);
                            this.attachTo(pane);
                        }
                    });
                    tab.tabRemoved$.subscribe(pane => {
                        this.logger.info('Pane closed', pane.title);
                    });
                });
            }
        });
        app.tabClosed$.subscribe(tab => {
            this.logger.info('Tab closed', tab.title);
        });
    }
    attachTo(tab) {
        this.index += 1;
        let id = this.index;
        this.logger.info(`Attached to ${tab.title}`);
        tab.frontendReady$.pipe(rxjs_1.first()).subscribe(() => {
            if (tab.frontend instanceof tabby_terminal_1.XTermFrontend) {
                this.logger.info('Got an xterm');
            }
        });
        tab.focused$.subscribe(() => {
            this.activePane = tab;
            this.logger.info('Active terminal tab:', tab.title);
            runCommand(`~/.fig/bin/fig hook keyboard-focus-changed tabby ${id}`, this.logger);
        });
        tab.blurred$.subscribe(() => {
            if (this.activePane === tab) {
                this.activePane = null;
            }
        });
    }
};
FigModule = __decorate([
    core_1.NgModule({
        imports: [
            tabby_core_1.default,
            tabby_terminal_1.default,
        ],
    }),
    __metadata("design:paramtypes", [tabby_core_1.LogService,
        tabby_core_1.AppService])
], FigModule);
exports.default = FigModule;


/***/ }),

/***/ "@angular/core":
/*!********************************!*\
  !*** external "@angular/core" ***!
  \********************************/
/***/ ((module) => {

module.exports = __WEBPACK_EXTERNAL_MODULE__angular_core__;

/***/ }),

/***/ "child_process":
/*!********************************!*\
  !*** external "child_process" ***!
  \********************************/
/***/ ((module) => {

module.exports = require("child_process");;

/***/ }),

/***/ "rxjs":
/*!***********************!*\
  !*** external "rxjs" ***!
  \***********************/
/***/ ((module) => {

module.exports = __WEBPACK_EXTERNAL_MODULE_rxjs__;

/***/ }),

/***/ "tabby-core":
/*!*****************************!*\
  !*** external "tabby-core" ***!
  \*****************************/
/***/ ((module) => {

module.exports = __WEBPACK_EXTERNAL_MODULE_tabby_core__;

/***/ }),

/***/ "tabby-terminal":
/*!*********************************!*\
  !*** external "tabby-terminal" ***!
  \*********************************/
/***/ ((module) => {

module.exports = __WEBPACK_EXTERNAL_MODULE_tabby_terminal__;

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		if(__webpack_module_cache__[moduleId]) {
/******/ 			return __webpack_module_cache__[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module is referenced by other modules so it can't be inlined
/******/ 	var __webpack_exports__ = __webpack_require__("./src/index.ts");
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});
//# sourceMappingURL=index.js.map