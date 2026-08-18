set nocompatible

syntax enable
filetype plugin indent on

set background=dark
colorscheme desert

set number
set relativenumber
set cursorline
set ruler
set showcmd
set scrolloff=5

set backspace=indent,eol,start

set expandtab
set tabstop=4
set shiftwidth=4
set softtabstop=4
set autoindent
set smartindent

set mouse=a

set ignorecase
set smartcase
set incsearch
set hlsearch

nnoremap <Esc> :nohlsearch<CR>

set wildmenu

nnoremap <C-h> <C-w>h
nnoremap <C-j> <C-w>j
nnoremap <C-k> <C-w>k
nnoremap <C-l> <C-w>l

augroup markdown_notes
    autocmd!
    autocmd FileType markdown setlocal wrap
    autocmd FileType markdown setlocal linebreak
    autocmd FileType markdown setlocal tabstop=2
    autocmd FileType markdown setlocal softtabstop=2
    autocmd FileType markdown setlocal shiftwidth=2
    autocmd FileType markdown setlocal expandtab
    autocmd FileType markdown setlocal conceallevel=0
    autocmd FileType markdown setlocal spell spelllang=en_us
    autocmd FileType markdown syntax clear
augroup END

autocmd FileType c,cpp,rust setlocal tabstop=4 shiftwidth=4 softtabstop=4 expandtab
