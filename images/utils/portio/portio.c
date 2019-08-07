/* .+

.identifier : $Id$
.context    : portio python extension
.title      : distutils setup
.kind	    : python source
.author     : Fabrizio Pollastri <f.pollastri@inrim.it>
.site	    : Torino - Italy
.creation   : 16-Feb-2006
.copyright  : (c) 2006-2012 Fabrizio Pollastri
              (c) 2012 Stjepan Henc <sthenc@gmail.com> python 3 porting.
.license    : GNU General Public License (see .copying below)

.copying

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

.- */

#include "Python.h"
#include <sys/io.h>

static PyObject *pio_outb(PyObject *self,PyObject *args)
{
  unsigned char data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"BH",&data,&port)) return NULL;
  outb(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outw(PyObject *self,PyObject *args)
{
  unsigned short int data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"HH",&data,&port)) return NULL;
  outw(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outl(PyObject *self,PyObject *args)
{
  unsigned int data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"IH",&data,&port)) return NULL;
  outl(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outb_p(PyObject *self,PyObject *args)
{
  unsigned char data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"BH",&data,&port)) return NULL;
  outb_p(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outw_p(PyObject *self,PyObject *args)
{
  unsigned short int data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"HH",&data,&port)) return NULL;
  outw_p(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outl_p(PyObject *self,PyObject *args)
{
  unsigned int data;
  unsigned short int port;
  if (!PyArg_ParseTuple(args,"IH",&data,&port)) return NULL;
  outl_p(data,port);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outsb(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  outsb(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outsw(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  outsw(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_outsl(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  outsl(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_inb(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned char data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inb(port);
  return Py_BuildValue("B",data);
}

static PyObject *pio_inw(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned short int data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inw(port);
  return Py_BuildValue("H",data);
}

static PyObject *pio_inl(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned int data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inl(port);
  return Py_BuildValue("I",data);
}

static PyObject *pio_inb_p(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned char data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inb_p(port);
  return Py_BuildValue("B",data);
}

static PyObject *pio_inw_p(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned short int data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inw_p(port);
  return Py_BuildValue("H",data);
}

static PyObject *pio_inl_p(PyObject *self,PyObject *args)
{
  unsigned short int port;
  unsigned int data;
  if (!PyArg_ParseTuple(args,"H",&port)) return NULL;
  data = inl_p(port);
  return Py_BuildValue("I",data);
}

static PyObject *pio_insb(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  insb(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_insw(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  insw(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_insl(PyObject *self,PyObject *args)
{
  unsigned short int port;
  void *string;
  unsigned long int count;
  if (!PyArg_ParseTuple(args,"Hsk",&port,&string,&count)) return NULL;
  insl(port,string,count);
  Py_INCREF(Py_None);
  return Py_None;
}

static PyObject *pio_ioperm(PyObject *self,PyObject *args)
{
  unsigned long int from;
  unsigned long int extent;
  int enable;
  int status;
  if (!PyArg_ParseTuple(args,"kki",&from,&extent,&enable)) return NULL;
  status = ioperm(from,extent,enable);
  if (status) status = errno;
  return Py_BuildValue("i",status);
}

static PyObject *pio_iopl(PyObject *self,PyObject *args)
{
  int level;
  int status;
  if (!PyArg_ParseTuple(args,"i",&level)) return NULL;
  status = iopl(level);
  if (status) status = errno;
  return Py_BuildValue("i",status);
}

/* List of methods defined in the module */

static struct PyMethodDef methods[] = {
  {"outb",(PyCFunction)pio_outb,METH_VARARGS,NULL},
  {"outw",(PyCFunction)pio_outw,METH_VARARGS,NULL},
  {"outl",(PyCFunction)pio_outl,METH_VARARGS,NULL},
  {"outsb",(PyCFunction)pio_outsb,METH_VARARGS,NULL},
  {"outsw",(PyCFunction)pio_outsw,METH_VARARGS,NULL},
  {"outsl",(PyCFunction)pio_outsl,METH_VARARGS,NULL},
  {"inb",(PyCFunction)pio_inb,METH_VARARGS,NULL},
  {"inw",(PyCFunction)pio_inw,METH_VARARGS,NULL},
  {"inl",(PyCFunction)pio_inl,METH_VARARGS,NULL},
  {"insb",(PyCFunction)pio_insb,METH_VARARGS,NULL},
  {"insw",(PyCFunction)pio_insw,METH_VARARGS,NULL},
  {"insl",(PyCFunction)pio_insl,METH_VARARGS,NULL},
  {"ioperm",(PyCFunction)pio_ioperm,METH_VARARGS,NULL},
  {"iopl",(PyCFunction)pio_iopl,METH_VARARGS,NULL},
  {NULL, (PyCFunction)NULL, 0, NULL}
};

/* module init function */

static char documentation[] =" PortIO, python low level port I/O for Linux x86\n\
\n\
PortIO is a Python front end to the low level functions provided by the\n\
C library on Linux 386 platforms for the hardware input and output ports:\n\
outb, outw, outl, outsb, outsw, outsl, outb_p, outw_p, outl_p, inb, inw,\n\
inl, insb, insw, insl, inb_p, inw_p, inl_p, ioperm, iopl.\n\
\n\
Before doing port I/O, it is mandatory to acquire proper privileges by\n\
calling ioperm or iopl. Otherwise you will get a segmentation fault.\n\
\n\
outb (data,port)\n\
  Output the byte data to the I/O address port.\n\
\n\
outb_p (data,port)\n\
  The same as outb, but waits for I/O completion.\n\
\n\
outw (data,port)\n\
  Output the 16 bit word data to the I/O address port.\n\
\n\
outw_p (data,port)\n\
  The same as outw, but waits for I/O completion.\n\
\n\
outl (data,port)\n\
  Output the 32 bit word data to the I/O address port.\n\
\n\
outl_p (data,port)\n\
  The same as outl, but waits for I/O completion.\n\
\n\
outsb (port,data,count)\n\
  Repeat count times the output of a byte to the I/O address port,\n\
  reading it from buffer of bytes starting at data and with length\n\
  count.\n\
\n\
outsw (port,data,count)\n\
  Repeat count times the output of a 16 bit word to the I/O address\n\
  port, reading it from buffer of 16 bit words starting at data and\n\
  with length count x 2.\n\
\n\
outsl (port,data,count)\n\
  Repeat count times the output of a 32 bit word to the I/O address\n\
  port, reading it from buffer of 32 bit words starting at data and\n\
  with length count x 4.\n\
\n\
inb (port)\n\
  Input a byte from the I/O address port and return it as integer.\n\
\n\
inb_p (port)\n\
  The same as inb, but waits for I/O completion.\n\
\n\
inw (port)\n\
  Input a 16 bit word from the I/O address port and return it as integer.\n\
\n\
inw_p (port)\n\
  The same as inw, but waits for I/O completion.\n\
\n\
inl (port)\n\
  Input a 32 bit word from the I/O address port and return it as integer.\n\
\n\
inl_p (port)\n\
  The same as inl, but waits for I/O completion.\n\
\n\
insb (port,data,count)\n\
  Repeat count times the input of a byte from the I/O address port\n\
  and write it to a buffer of bytes starting at data and with length\n\
  count bytes.\n\
\n\
insw (port,data,count)\n\
  Repeat count times the input of a 16 bit word from the I/O address\n\
  port and write it to a buffer of 16 bit words starting at data\n\
  and with length count x 2 bytes.\n\
\n\
insl (port,data,count)\n\
  Repeat count times the input of a 32 bit word from the I/O address\n\
  port and write it to a buffer of 32 bit words starting at data\n\
  and with length count x 4 bytes.\n\
\n\
ioperm (from,extent,enable)\n\
  Set port access permission starting from address from for extent\n\
  bytes. If the enable is True, access is enabled, otherwise is disabled.\n\
  On success, zero is returned. On error, the errno code is returned.\n\
  The use of ioperm requires root privileges.\n\
\n\
  Only the first 0x3ff I/O ports can be specified in this manner. To gain\n\
  access to any I/O port in the whole (0x0000-0xffff) address range, use\n\
  the iopl function. \n\
\n\
iopl (level)\n\
  Set the I/O privilege level of the current process. When level is 3\n\
  access is granted to any I/O port.\n\
  On success, zero is returned. On error, the errno code is returned.\n\
  The use of iopl requires root privileges.\n\
\n\
";

/* module init for both python < 3.0 and >= 3.0 */

static PyObject *Error;

#if PY_MAJOR_VERSION < 3	/* is python < 3.0 */

PyMODINIT_FUNC
initportio(void)
{
  PyObject *m;
  m = Py_InitModule3("portio", methods, documentation);
  if(m == NULL)
    return;

  /* add module specific exception */
  Error = PyErr_NewException("portio.error", NULL, NULL);
  Py_INCREF(Error);
  PyModule_AddObject(m, "error", Error);
}

#else				/* is python >= 3.0 */

static struct PyModuleDef portiomodule = {
   PyModuleDef_HEAD_INIT, "portio", documentation, -1, methods
};

PyMODINIT_FUNC
PyInit_portio(void)
{
  PyObject *m;
  m = PyModule_Create(&portiomodule);
  if(m == NULL)
    return NULL;

  /* add module specific exception */
  Error = PyErr_NewException("portio.error", NULL, NULL);
  Py_INCREF(Error);
  PyModule_AddObject(m, "error", Error);
	
  return m;
}

#endif

/* end */
