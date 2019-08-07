#!/usr/bin/python
# .+
#
# .context    : portio python extension
# .title      : distutils setup
# .kind	      : python source
# .author     : Fabrizio Pollastri <f.pollastri@inrim.it>
# .site	      : Torino - Italy
# .creation   :	16-Feb-2006
# .copyright  :	(c) 2006-2012 Fabrizio Pollastri
#               (c) 2012 Stjepan Henc <shenc@gmail.com> python 3 porting.
# .license    : GNU General Public License (see .copying below)
#
# .copying
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
# 
# .-

from distutils.core import setup, Extension
import os
import os.path
import re
import string
import sys

classifiers = """\
Development Status :: 4 - Beta
Intended Audience :: Developers
License :: OSI Approved :: GNU General Public License (GPL)
Programming Language :: Python
Programming Language :: Python :: 3
Topic :: System :: Hardware
Topic :: Software Development :: Libraries :: Python Modules
Operating System :: POSIX :: Linux
"""

# check for proper python version
if sys.version < '2.6':
  print('\nportio 0.5 requires python >= 2.6 , found python', \
    sys.version.split()[0], ', exiting...')
  sys.exit()

# if python docutils are installed, generate HTML documentation.
if os.path.exists('/usr/bin/rst-buildhtml'):
  os.system('/usr/bin/rst-buildhtml')

readme = open('index.rst').read()	# read in documentation file
readme = readme.replace('**','')	# remove restructuredtext markup

# split documentation by text titles and put each title and its text body
# into a dictionary: title is the key, text body is the value.
readme_split = re.split('\n(.*?)\n====+?\n\n',readme)
i =  3
readme_db = {}
while i < len(readme_split):
  readme_db[readme_split[i]] = readme_split[i + 1]
  i += 2

main_title = readme_split[1] + '\n'	# main title goes alone

# if not present, generate portio.c source file.
# Replace template tag "DOCUMENTATION" into C source with documentation
# text read from README. Add backslash iand new line at end of each line
# (C multiline string syntax).
if not os.path.exists('portio.c'):
  doc = main_title + '\n' + readme_db["Module reference"]
  doc = doc.replace('\n','\\n\\\n')
  source = open('portio.c.in').read()
  source = source % {'DOCUMENTATION':doc}
  open('portio.c','w').write(source)

# if not present, generate toggle.py source file extracting it from README
if not os.path.exists('toggle.py'):
  source = readme_db["Usage example"]
  source = source.split('::')[1]
  print(source)

module = Extension(
  'portio',
  define_macros = [('MAJOR_VERSION', '0'),('MINOR_VERSION', '5')],
  include_dirs = ['/usr/local/include'],
  libraries = [],
  library_dirs = ['/usr/lib'],
  sources = ['portio.c'])

setup (
  name = 'portio',
  version = '0.5',
  author = 'Fabrizio Pollastri',
  author_email = 'f.pollastri@inrim.it',
  maintainer = 'Fabrizio Pollastri',
  maintainer_email = 'f.pollastri@inrim.it',
  url = 'http://portio.inrim.it',
  license = 'http://www.gnu.org/licenses/gpl.txt',
  platforms = ['Linux'],
  description = main_title,
  classifiers = filter(None, classifiers.split("\n")),
  long_description =  readme_db["Module reference"],
  ext_modules = [module])

# cleanup
try:
  os.remove('MANIFEST')
except:
  pass

#### END
