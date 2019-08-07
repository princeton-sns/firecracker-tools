
================================================
 PortIO, python low level port I/O for Linux x86
================================================


What is
=======

PortIO is a Python wrapper for the port I/O macros like **outb, inb**, etc. 
provided by the C library on Linux x86 platforms. Both python 2 and 3 are
supported. This module is useful when a general pourpose port I/O at the low
level is needed. Programmers that want to perform I/O on the parallel port
at an higher level, will be better satisfied by the
`pyParallel <http://pyserial.sourceforge.net/pyparallel.html>`_ module.
A similar module `Ioport <http://www.hare.demon.co.uk/ioport/ioport.html>`_
has inspired the writing of PortIO.

PortIO is released under the `GNU General Public License
<http://www.gnu.org/licenses/gpl.txt>`_.

*At present, version 0.5, PortIO is in beta status. Any debugging aid is
welcome.*

For any question, suggestion, contribution contact the author
`Fabrizio Pollastri` <f.pollastri_a_t_inrim.it>.

The PortIO web site is hosted at http://portio.inrim.it/.


Usage example
=============

This sample program toggle on and off all the data lines of the parallel port
lp0 with a 6 seconds period. Note the check for root privileges before
the call to **ioperm** to acquire the proper I/O permissions for the involved
ports.

.. include:: toggle.py
   :start-after: .-
   :end-before: #### END
   :literal:

Download the sample program `toggle.py <./toggle.py>`_


Module reference
================

PortIO is a Python front end to the low level functions provided by the
C library on Linux 386 platforms for the hardware input and output ports:
**outb, outw, outl, outsb, outsw, outsl, outb_p, outw_p, outl_p, inb, inw,
inl, insb, insw, insl, inb_p, inw_p, inl_p, ioperm, iopl**.

Before doing port I/O, it is mandatory to acquire proper privileges by
calling **ioperm** or **iopl**. Otherwise you will get a segmentation fault.

**outb (data,port)**
  Output the byte **data** to the I/O address **port**.

**outb_p (data,port)**
  The same as **outb**, but waits for I/O completion.

**outw (data,port)**
  Output the 16 bit word **data** to the I/O address **port**.

**outw_p (data,port)**
  The same as **outw**, but waits for I/O completion.

**outl (data,port)**
  Output the 32 bit word **data** to the I/O address **port**.

**outl_p (data,port)**
  The same as **outl**, but waits for I/O completion.

**outsb (port,data,count)**
  Repeat **count** times the output of a byte to the I/O address **port**,
  reading it from buffer of bytes starting at **data** and with length
  **count**.

**outsw (port,data,count)**
  Repeat **count** times the output of a 16 bit word to the I/O address
  **port**, reading it from buffer of 16 bit words starting at **data** and
  with length **count** x 2.

**outsl (port,data,count)**
  Repeat **count** times the output of a 32 bit word to the I/O address
  **port**, reading it from buffer of 32 bit words starting at **data** and
  with length **count** x 4.

**inb (port)**
  Input a byte from the I/O address **port** and return it as integer.

**inb_p (port)**
  The same as **inb**, but waits for I/O completion.

**inw (port)**
  Input a 16 bit word from the I/O address **port** and return it as integer.

**inw_p (port)**
  The same as **inw**, but waits for I/O completion.

**inl (port)**
  Input a 32 bit word from the I/O address **port** and return it as integer.

**inl_p (port)**
  The same as **inl**, but waits for I/O completion.

**insb (port,data,count)**
  Repeat **count** times the input of a byte from the I/O address **port**
  and write it to a buffer of bytes starting at **data** and with length
  **count** bytes.

**insw (port,data,count)**
  Repeat **count** times the input of a 16 bit word from the I/O address
  **port** and write it to a buffer of 16 bit words starting at **data**
  and with length **count** x 2 bytes.

**insl (port,data,count)**
  Repeat **count** times the input of a 32 bit word from the I/O address
  **port** and write it to a buffer of 32 bit words starting at **data**
  and with length **count** x 4 bytes.

**ioperm (from,extent,enable)**
  Set port access permission starting from address **from** for **extent**
  bytes. If the **enable** is True, access is enabled, otherwise is disabled.
  On success, zero is returned. On error, the errno code is returned.
  The use of ioperm requires root privileges.

  Only the first 0x3ff I/O ports can be specified in this manner. To gain
  access to any I/O port in the whole (0x0000-0xffff) address range, use
  the iopl function. 

**iopl (level)**
  Set the I/O privilege level of the current process. When **level** is 3
  access is granted to any I/O port.
  On success, zero is returned. On error, the errno code is returned.
  The use of iopl requires root privileges.


Requirements
============

A **linux on an X86 architecture**.

To run the code, **Python 2.6 or later** or **Python 3.0 or later** must
already be installed.  The latest release is recommended.  Python is
available from http://www.python.org/.


Installation
============

With easy_install
-----------------

1. Open a shell.

2. Get root privileges and install the package. Command: ::

        easy_install portio


From tarball
------------

Download PortIO tarball from http://portio.inrim.it/portio-0.5.tar.gz .

The first step is to expand the ``.tgz`` archive in a temporary
directory (**not** directly in Python's ``site-packages``).  It
contains a distutils setup file "setup.py". 

1. Open a shell.

2. Unpack the tarball in a temporary directory (**not** directly in
   Python's ``site-packages``). Command: ::

        tar zxf portio-X.Y.tar.gz

   X and Y are the major and minor version numbers of the tarball.

2. Go to the directory created by expanding the tarball. Command: ::

       cd portio-X.Y

3. Get root privileges and install the package. Command: ::

       su
       (enter root password)
       python setup.py install

   If the python executable isn't on your path, you'll have to specify
   the complete path, such as /usr/local/bin/python.


Changelog
=========

**Portio 0.5 released 25-Oct-2012**

* Porting to python 3 (also contributed by Stjepan Henc <sthenc_a_t_gmail.com>.

**Portio 0.4 released 25-Aug-2009**

* Fixed some argument type mismatch in I/O macros.

* Upgraded PyArg_ParseTuple format strings with the new "unsigned" formats
  available from python 2.3 . So portio now requires python version => 2.3 .

**Portio 0.3 released 21-May-2009**

* Fixed missing documentation files.

**Portio 0.2 released 11-Nov-2008**

* Added return of status code for **ioperm** and **iopl**.

* Fixed invalid argument type for **ioperm**.

* Updated **toggle.py** example with **ioperm** error check.

* Generated documentation with Sphinx.

**Portio 0.1 released 23-Feb-2006**

* First release.


Credits
=======

* Thanks to Stjepan Henc <sthenc_a_t_gmail.com> for his contribution to python 3 porting.


----

Copyright 2006-2012 by `Fabrizio Pollastri` <f.pollastri_a_t_inrim.it>


..
   Local Variables:
   mode: indented-text
   indent-tabs-mode: nil
   sentence-end-double-space: t
   fill-column: 70
   End:
