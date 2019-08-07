#!/usr/bin/python
# .+
#
# .context    : PortIO
# .title      : Toggle parallel port lines
# .kind	      : command shell
# .author     : Fabrizio Pollastri <f.pollastri@inrim.it>
# .site	      : Torino - Italy
# .creation   :	13-Nov-2008
# .copyright  :	(c) 2008-2012 Fabrizio Pollastri
#               (c) 2012 Stjepan Henc <sthenc@gmail.com> python 3 porting.
# .license    : GNU General Public License (see below)
#
# This file is part of "PortIO, python low level I/O for Linux x86".
#
# PortIO is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 3 of the License, or
# (at your option) any later version.
#
# PortIO is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# .-

import sys, time, os
import portio

# check for root privileges
if os.getuid():
  print('You need to be root! Exiting.')
  sys.exit()

# acquire permission for I/O on lp0
status = portio.ioperm(0x378, 1, 1)
if status:
  print('ioperm:',os.strerror(status))
  sys.exit()

# toggle forever the data lines of lp0
data = 0
while 1:
  lp0in = portio.inb(0x378)
  portio.outb(data,0x378) 
  print('read %x from lp0, written %x to lp0' % (lp0in,data))
  data = ~data & 0xff
  time.sleep(3)

#### END
