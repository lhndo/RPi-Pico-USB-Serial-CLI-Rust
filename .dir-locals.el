;; https://rust-analyzer.github.io/book/configuration.html
((rust-mode
  (eglot-workspace-configuration
   (:rust-analyzer
    
    (cargo target "thumbv6m-none-eabi")
    (cargo allTargets :json-false)
    (check allTargets :json-false)

    (procMacro enable t)
    (check buildScripts enable t)

    (cfg setTest :json-false)
    (cfg test :json-false)

            
    ))))

