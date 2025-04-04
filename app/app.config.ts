export default defineAppConfig({
  ui: {
    primary: 'sky',
    gray: 'cool',
    avatar: {
      default: {
        icon: 'i-ph-image'
      }
    },
    button: {
      default: {
        loadingIcon: 'i-ph-spinner'
      }
    },
    input: {
      default: {
        loadingIcon: 'i-ph-spinner'
      }
    },
    select: {
      default: {
        loadingIcon: 'i-ph-spinner',
        trailingIcon: 'i-ph-caret-down'
      }
    },
    selectMenu: {
      default: {
        selectedIcon: 'i-ph-check'
      }
    },
    notification: {
      default: {
        closeButton: {
          icon: 'i-ph-x'
        }
      }
    },
    commandPalette: {
      default: {
        icon: 'i-ph-magnifying-glass',
        loadingIcon: 'i-ph-spinner',
        selectedIcon: 'i-ph-check',
        emptyState: {
          icon: 'i-ph-magnifying-glass'
        },
        closeButton: {
          icon: 'i-ph-x'
        }
      }
    },
    table: {
      default: {
        sortAscIcon: 'i-ph-sort-ascending',
        sortDescIcon: 'i-ph-sort-descending',
        sortButton: {
          icon: 'i-ph-list'
        },
        loadingState: {
          icon: 'i-ph-spinner'
        },
        emptyState: {
          icon: 'i-ph-database'
        }
      }
    },
    pagination: {
      default: {
        prevButton: {
          icon: 'i-ph-arrow-left'
        },
        nextButton: {
          icon: 'i-ph-arrow-right'
        }
      }
    },
    card: {
      rounded: 'rounded-xl'
    },
    tooltip: {
      background: '!bg-background',
      popper: {
        strategy: 'absolute'
      }
    },
    breadcrumb: {
      divider: {
        base: 'w-4 h-4'
      },
      default: {
        divider: 'i-ph-caret-right'
      }
    },
    // `@nuxt/ui-pro` specific
    variables: {
      dark: {
        background: 'var(--color-gray-950)'
      }
    },
    icons: {
      // dark: 'i-ph-moon',
      // light: 'i-ph-sun',
      search: 'i-ph-magnifying-glass',
      external: 'i-ph-arrow-up-right',
      chevron: 'i-ph-caret-down',
      hash: 'i-ph-hash'
    },
    header: {
      wrapper: 'lg:mb-0 lg:border-0',
      links: {
        trailingIcon: {
          base: 'w-4 h-4'
        },
        popover: {
          popper: {
            strategy: 'absolute'
          },
          ui: {
            width: 'w-[16rem]'
          }
        }
      },
      popover: {
        links: {
          active: 'dark:bg-gray-950/50',
          inactive: 'dark:hover:bg-gray-950/50'
        }
      },
      button: {
        icon: {
          open: 'i-ph-list',
          close: 'i-ph-x'
        }
      }
    },
    navigation: {
      accordion: {
        button: {
          trailingIcon: {
            base: 'w-4 h-4'
          }
        }
      }
    },
    page: {
      card: {
        to: 'dark:hover:bg-gray-900/50'
      },
      hero: {
        wrapper: 'lg:py-24'
      }
    },
    content: {
      search: {
        fileIcon: {
          name: 'i-ph-file-text'
        }
      },
      toc: {
        button: {
          trailingIcon: {
            base: 'w-4 h-4'
          }
        }
      },
      surround: {
        icon: {
          prev: 'i-ph-arrow-left',
          next: 'i-ph-arrow-right'
        }
      },
      collapsible: {
        button: {
          icon: {
            base: 'w-3 h-3'
          }
        }
      },
      prose: {
        code: {
          button: {
            icon: {
              copy: 'i-ph-copy',
              copied: 'i-ph-check-square'
            }
          },
          icon: {
            terminal: 'i-ph-terminal-window'
          }
        }
      }
    },
    footer: {
      top: {
        wrapper: 'border-t border-gray-200 dark:border-gray-800',
        container: 'py-8 lg:py-16'
      },
      bottom: {
        wrapper: 'border-t border-gray-200 dark:border-gray-800'
      }
    },
    landing: {
      hero: {
        title: 'text-2xl sm:text-5xl',
      }

    },
  }
})