# Simple test script
import pexpect


# Start QEMU
child = pexpect.spawn('make run',encoding='utf-8')
fout = open('log.txt','w')
child.logfile = fout
# Wait for the prompt
child.expect('banscii>',timeout=1)
# Try hello world
child.sendline('Hello\r')
child.expect('Sending some data: PING',timeout=1)
child.sendline('World\r')
child.expect('Sending some data: PONG',timeout=1)
# Escape sequence
child.sendcontrol('A')
child.send('x')
# Termination confirmation
child.expect('QEMU: Terminated',timeout=1)

print("Test Succeeded!")
