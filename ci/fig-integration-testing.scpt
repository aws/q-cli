FasdUAS 1.101.10   ��   ��    k             p         ������ 0 
passphrase  ��      	  l     ��������  ��  ��   	  
  
 i         I      �������� &0 promptforpassword promptForPassword��  ��    k            p         ������ 0 
passphrase  ��        l     ��������  ��  ��        r         I    ��  
�� .sysodlogaskr        TEXT  m        �   Z P l e a s e   e n t e r   a   p a s s p h r a s e   t o   u s e   t h i s   s c r i p t .  ��  
�� 
dtxt  m       �      ��   !
�� 
disp   m    ��
�� stic     ! �� " #
�� 
btns " J    
 $ $  % & % m     ' ' � ( (  C a n c e l &  )�� ) m     * * � + +  C o n t i n u e��   # �� , -
�� 
dflt , m     . . � / /  C o n t i n u e - �� 0��
�� 
htxt 0 m    ��
�� boovtrue��    o      ���� 0 response     1�� 1 r     2 3 2 n     4 5 4 1    ��
�� 
ttxt 5 o    ���� 0 response   3 o      ���� 0 
passphrase  ��     6 7 6 l     ��������  ��  ��   7  8 9 8 i     : ; : I      �� <���� $0 changeloginshell changeLoginShell <  =�� = o      ���� 0 
executable  ��  ��   ; k      > >  ? @ ? p       A A ������ 0 pass  ��   @  B C B l     ��������  ��  ��   C  D E D l     �� F G��   F W Q do shell script "chsh -s " & executable & " $USER" with administrator privileges    G � H H �   d o   s h e l l   s c r i p t   " c h s h   - s   "   &   e x e c u t a b l e   &   "   $ U S E R "   w i t h   a d m i n i s t r a t o r   p r i v i l e g e s E  I J I I    �� K��
�� .sysoexecTEXT���     TEXT K b     	 L M L b      N O N b      P Q P b      R S R m      T T � U U  e c h o   ' S o    ���� 0 
passphrase   Q m     V V � W W ( '   |   s u d o   - S   c h s h   - s   O o    ���� 0 
executable   M m     X X � Y Y    $ U S E R��   J  Z [ Z l   ��������  ��  ��   [  \ ] \ l   �� ^ _��   ^   Update shell in Hyper    _ � ` ` ,   U p d a t e   s h e l l   i n   H y p e r ]  a�� a I   �� b��
�� .sysoexecTEXT���     TEXT b b     c d c b     e f e m     g g � h h 8 s e d   " s + s h e l l :   ' . * ' , + s h e l l :   ' f o    ���� 0 
executable   d m     i i � j j | ' , + "     ~ / . h y p e r . j s   >   / t m p / t m p f i l e   ;   m v   / t m p / t m p f i l e   ~ / . h y p e r . j s��  ��   9  k l k l     ��������  ��  ��   l  m n m l     �� o p��   o z t https://apple.stackexchange.com/questions/230983/how-do-you-do-command-line-video-screen-capture-on-os-x-with-libav    p � q q �   h t t p s : / / a p p l e . s t a c k e x c h a n g e . c o m / q u e s t i o n s / 2 3 0 9 8 3 / h o w - d o - y o u - d o - c o m m a n d - l i n e - v i d e o - s c r e e n - c a p t u r e - o n - o s - x - w i t h - l i b a v n  r s r i     t u t I      �� v���� 0 record_command_with_ffmpeg   v  w x w o      ���� 0 sequence_name   x  y�� y o      ���� 0 duration  ��  ��   u I    �� z��
�� .sysoexecTEXT���     TEXT z b     	 { | { b      } ~ } b       �  b      � � � m      � � � � � h f f m p e g   - f   a v f o u n d a t i o n   - p i x _ f m t   y u y v 4 2 2   - i   " 1 : 1 "   - t   � o    ���� 0 duration   � m     � � � � � V   - v f   c r o p = 5 5 0 , 4 0 0 : 5 0 : 5 0   - r   6 0   ~ / t e s t / d e b u g / ~ o    ���� 0 sequence_name   | m     � � � � � . . m p 4   >   / d e v / n u l l   2 > & 1   &��   s  � � � l     ��������  ��  ��   �  � � � i     � � � I      �� ����� 0 record_command   �  � � � o      ���� 0 sequence_name   �  ��� � o      ���� 0 duration  ��  ��   � k      � �  � � � I    �� ���
�� .sysoexecTEXT���     TEXT � b     	 � � � b      � � � b      � � � b      � � � m      � � � � � ( s c r e e n c a p t u r e   - v   - V   � o    ���� 0 duration   � m     � � � � � H   - T   0   - R   5 0 , 5 0 , 5 5 0 , 4 0 0   ~ / t e s t / d e b u g / � o    ���� 0 sequence_name   � m     � � � � � . . m o v   >   / d e v / n u l l   2 > & 1   &��   �  � � � I   �� ���
�� .sysodelanull��� ��� nmbr � m    ���� ��   �  ��� � l   ��������  ��  ��  ��   �  � � � l     ��������  ��  ��   �  � � � i     � � � I      �� ����� 0 generate_collage   �  ��� � o      ���� 	0 shell  ��  ��   � I    	�� ���
�� .sysoexecTEXT���     TEXT � b      � � � b      � � � m      � � � � � $ e x p o r t   T E S T _ S H E L L = � o    ���� 	0 shell   � m     � � � � �T   & &   c d   ~ / t e s t / d e b u g   & &   / o p t / h o m e b r e w / b i n / f f m p e g   - i   a l a c r i t t y - $ T E S T _ S H E L L . m o v   - i   h y p e r - $ T E S T _ S H E L L . m o v   - i   i t e r m - $ T E S T _ S H E L L . m o v   - i   t e r m i n a l - $ T E S T _ S H E L L . m o v   - f i l t e r _ c o m p l e x   " [ 0 : v ] [ 1 : v ] h s t a c k = i n p u t s = 2 [ t o p ] ;   [ 2 : v ] [ 3 : v ] h s t a c k = i n p u t s = 2 [ b o t t o m ] ;   [ t o p ] [ b o t t o m ] v s t a c k = i n p u t s = 2 [ v ] "   - m a p   " [ v ] "   $ T E S T _ S H E L L . m p 4��   �  � � � l     ��������  ��  ��   �  � � � i     � � � I      �� ����� 0 generate_horizontal_collage   �  ��� � o      ���� 	0 shell  ��  ��   � I    	�� ���
�� .sysoexecTEXT���     TEXT � b      � � � b      � � � m      � � � � � $ e x p o r t   T E S T _ S H E L L = � o    ���� 	0 shell   � m     � � � � ��   & &   c d   ~ / t e s t / d e b u g   & &   / o p t / h o m e b r e w / b i n / f f m p e g   - i   h y p e r - $ T E S T _ S H E L L . m o v   - i   i t e r m - $ T E S T _ S H E L L . m o v   - i   t e r m i n a l - $ T E S T _ S H E L L . m o v   - i   a l a c r i t t y - $ T E S T _ S H E L L . m o v   - f i l t e r _ c o m p l e x   h s t a c k = i n p u t s = 4   $ T E S T _ S H E L L . m p 4��   �  � � � l     ��������  ��  ��   �  � � � i     � � � I      �� ����� 0 basename   �  ��� � 1      ��
�� 
ppth��  ��   � I    	�� ���
�� .sysoexecTEXT���     TEXT � b      � � � b      � � � m      � � � � � & / u s r / b i n / b a s e n a m e   ' � 1    ��
�� 
ppth � m     � � � � �  '��   �  � � � l     ��������  ��  ��   �  � � � i     � � � I      �� ����� 0 typecommand typeCommand �  � � � o      ���� 0 cmd   �  ��� � o      ���� 0 d  ��  ��   � k      � �  � � � O     
 � � � I   	�� ���
�� .prcskprsnull���     ctxt � o    ���� 0 cmd  ��   � m      � ��                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��   �  ��� � I   � ��~
� .sysodelanull��� ��� nmbr � o    �}�} 0 d  �~  ��   �  � � � l     �|�{�z�|  �{  �z   �  � � � i     # � � � I      �y ��x�y 0 
pressenter 
pressEnter �  ��w � o      �v�v 0 d  �w  �x   � k      � �  � � � O     
 � � � I   	�u ��t
�u .prcskcodnull���     **** � m    �s�s $�t   � m      � ��                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��   �  ��r � I   �q ��p
�q .sysodelanull��� ��� nmbr � o    �o�o 0 d  �p  �r   �    l     �n�m�l�n  �m  �l    i   $ ' I      �k�j�k 0 pressescape pressEscape �i o      �h�h 0 d  �i  �j   k      	
	 O     
 I   	�g�f
�g .prcskcodnull���     **** m    �e�e 5�f   m     �                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  
 �d I   �c�b
�c .sysodelanull��� ��� nmbr o    �a�a 0 d  �b  �d    l     �`�_�^�`  �_  �^    i   ( + I      �]�\�]  0 executecommand executeCommand  o      �[�[ 0 cmd   �Z o      �Y�Y 0 d  �Z  �\   k       I     �X�W�X 0 typecommand typeCommand   o    �V�V 0 cmd    !�U! m    �T�T  �U  �W   "�S" I    �R#�Q�R 0 
pressenter 
pressEnter# $�P$ o   	 
�O�O 0 d  �P  �Q  �S   %&% l     �N�M�L�N  �M  �L  & '(' i   , /)*) I      �K+�J�K 0 	clearline 	clearLine+ ,�I, o      �H�H 0 d  �I  �J  * k     -- ./. O     010 I   �G23
�G .prcskprsnull���     ctxt2 m    44 �55  u3 �F6�E
�F 
faal6 m    �D
�D eMdsKctl�E  1 m     77�                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  / 8�C8 I   �B9�A
�B .sysodelanull��� ��� nmbr9 o    �@�@ 0 d  �A  �C  ( :;: l     �?�>�=�?  �>  �=  ; <=< i   0 3>?> I      �<@�;�<  0 launchterminal launchTerminal@ ABA 1      �:
�: 
pnamB CDC o      �9�9 0 ps  D E�8E o      �7�7 0 opennewwindow openNewWindow�8  �;  ? k     VFF GHG O    IJI I   �6�5�4
�6 .miscactvnull��� ��� null�5  �4  J 4     �3K
�3 
cappK 1    �2
�2 
pnamH LML I   �1N�0
�1 .sysodelanull��� ��� nmbrN m    �/�/ �0  M OPO Z    /QR�.�-Q o    �,�, 0 opennewwindow openNewWindowR O    +STS k    *UU VWV I   $�+XY
�+ .prcskprsnull���     ctxtX m    ZZ �[[  nY �*\�)
�* 
faal\ m     �(
�( eMdsKcmd�)  W ]�'] I  % *�&^�%
�& .sysodelanull��� ��� nmbr^ m   % &�$�$ �%  �'  T m    __�                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  �.  �-  P `�#` O  0 Vaba O   4 Ucdc O   ; Tefe k   B Sgg hih r   B Jjkj J   B Fll mnm m   B C�"�" 2n o�!o m   C D� �  2�!  k 1   F I�
� 
posni p�p r   K Sqrq J   K Oss tut m   K L��&u v�v m   L M����  r 1   O R�
� 
ptsz�  f 4   ; ?�w
� 
cwinw m   = >�� d 4   4 8�x
� 
prcsx o   6 7�� 0 ps  b m   0 1yy�                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  �#  = z{z l     ����  �  �  { |}| i   4 7~~ I      ���� *0 closeterminalwindow closeTerminalWindow�  �   O     ��� k    �� ��� I   ���
� .prcskprsnull���     ctxt� m    �� ���  w� ���
� 
faal� m    �
� eMdsKcmd�  � ��� I   �
��	
�
 .sysodelanull��� ��� nmbr� m    �� ?�      �	  �  � m     ���                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  } ��� l     ����  �  �  � ��� i   8 ;��� I      ���� 0 quitterminal quitTerminal� ��� 1      �
� 
pnam�  �  � O     ��� I   �� ��
� .aevtquitnull��� ��� null�   ��  � 4     ���
�� 
capp� 1    ��
�� 
pnam� ��� l     ��������  ��  ��  � ��� i   < ?��� I      ������� 0 newtab newTab� ���� o      ���� 0 d  ��  ��  � O     ��� k    �� ��� I   ����
�� .prcskprsnull���     ctxt� m    �� ���  t� �����
�� 
faal� m    ��
�� eMdsKcmd��  � ���� I   �����
�� .sysodelanull��� ��� nmbr� o    ���� 0 d  ��  ��  � m     ���                                                                                  sevs  alis    \  Macintosh HD                   BD ����System Events.app                                              ����            ����  
 cu             CoreServices  0/:System:Library:CoreServices:System Events.app/  $  S y s t e m   E v e n t s . a p p    M a c i n t o s h   H D  -System/Library/CoreServices/System Events.app   / ��  � ��� l     ��������  ��  ��  � ��� i   @ C��� I      �������� 0 setup setUp��  ��  � k     �� ��� I     �������� &0 promptforpassword promptForPassword��  ��  � ��� I   �����
�� .sysoexecTEXT���     TEXT� m    �� ��� v m k d i r   - p   ~ / t e s t / d e b u g   & &   r m   ~ / t e s t / d e b u g / *   | |   e c h o   n o   f i l e s��  � ��� I   �����
�� .sysoexecTEXT���     TEXT� m    �� ��� < m k d i r   - p   $ T M P D I R / f i g _ t e s t s / d i r��  � ��� I   �����
�� .sysoexecTEXT���     TEXT� m    �� ��� � c d   $ T M P D I R / f i g _ t e s t s   & &   t o u c h   a . t x t   b . j s   c . c   | |   e c h o   f i l e s   a l r e a d y   e x i s t��  � ���� I   �����
�� .sysoexecTEXT���     TEXT� m    �� ��� � c d   $ T M P D I R / f i g _ t e s t s / d i r   & &   t o u c h   1 . p y   2 . j s o n   3 . s w i f t   | |   e c h o   f i l e s   a l r e a d y   e x i s t��  ��  � ��� l     ��������  ��  ��  � ��� i   D G��� I      �������� 0 teardown tearDown��  ��  � k     )�� ��� I     ������� 0 generate_horizontal_collage  � ���� m    �� ���  f i s h��  ��  � ��� I    ������� 0 generate_horizontal_collage  � ���� m    	�� ���  z s h��  ��  � ��� I    ������� 0 generate_horizontal_collage  � ���� m    �� ���  b a s h��  ��  � ��� l   ��������  ��  ��  � ��� I   �����
�� .sysoexecTEXT���     TEXT� m    �� ��� � c d   ~ / t e s t / d e b u g   & &   / o p t / h o m e b r e w / b i n / f f m p e g   - i   z s h . m p 4   - i   b a s h . m p 4   - i   f i s h . m p 4   - f i l t e r _ c o m p l e x   v s t a c k = i n p u t s = 3   f i n a l . m p 4��  � ��� l   ��������  ��  ��  � ��� I    �����
�� .sysoexecTEXT���     TEXT� m    �� ��� " o p e n   ~ / t e s t / d e b u g��  � ��� l  ! !��������  ��  ��  � ��� I   ! '������� $0 changeloginshell changeLoginShell� ���� m   " #�� ���  / b i n / z s h��  ��  � ���� l  ( (��������  ��  ��  ��  � ��� l     ��������  ��  ��  � ��� i   H K��� I      �� ���� 0 testsequence testSequence   o      ���� 0 filename   �� o      ���� 0 duration  ��  ��  � k     U  I     ������  0 executecommand executeCommand 	 m    

 � 
 c l e a r	 �� m     ?�      ��  ��    I    ������  0 executecommand executeCommand  b   	  m   	 
 � H :   S t a r t i n g   a u t o m a t e d   t e s t   s e q u e n c e :   o   
 ���� 0 filename   �� m     ?�      ��  ��    I    ������  0 executecommand executeCommand  m     � ( c d   $ T M P D I R / f i g _ t e s t s  ��  m    !! ?�      ��  ��   "#" l   ��������  ��  ��  # $%$ l   ��������  ��  ��  % &'& I    !��(���� 0 record_command  ( )*) o    ���� 0 filename  * +��+ o    ���� 0 duration  ��  ��  ' ,-, l  " "��������  ��  ��  - ./. l  " "��01��  0 + % CHECK: is working directory correct?   1 �22 J   C H E C K :   i s   w o r k i n g   d i r e c t o r y   c o r r e c t ?/ 343 l  " "��56��  5 , & CHECK: are shell suggestions loading?   6 �77 L   C H E C K :   a r e   s h e l l   s u g g e s t i o n s   l o a d i n g ?4 898 I   " )��:���� 0 typecommand typeCommand: ;<; m   # $== �>>  c d  < ?��? m   $ %@@ ?�      ��  ��  9 ABA l  * *��������  ��  ��  B CDC l  * *��EF��  E + % CHECK: are the suggestions filtered?   F �GG J   C H E C K :   a r e   t h e   s u g g e s t i o n s   f i l t e r e d ?D HIH I   * 1��J���� 0 typecommand typeCommandJ KLK m   + ,MM �NN  dL O��O m   , -���� ��  ��  I PQP l  2 2��������  ��  ��  Q RSR l  2 2��TU��  T ) # CHECK: does simple insertion work?   U �VV F   C H E C K :   d o e s   s i m p l e   i n s e r t i o n   w o r k ?S WXW I   2 8��Y���� 0 
pressenter 
pressEnterY Z��Z m   3 4���� ��  ��  X [\[ l  9 9��������  ��  ��  \ ]^] l  9 9��_`��  _   Run command in terminal   ` �aa 0   R u n   c o m m a n d   i n   t e r m i n a l^ bcb I   9 ?�d�~� 0 
pressenter 
pressEnterd e�}e m   : ;ff ?�      �}  �~  c ghg l  @ @�|�{�z�|  �{  �z  h iji l  @ @�ykl�y  k + % CHECK: is working directory correct?   l �mm J   C H E C K :   i s   w o r k i n g   d i r e c t o r y   c o r r e c t ?j non I   @ G�xp�w�x 0 typecommand typeCommandp qrq m   A Bss �tt  l s  r u�vu m   B C�u�u �v  �w  o vwv l  H H�t�s�r�t  �s  �r  w xyx l  H H�qz{�q  z , & CHECK: does escape dismiss the popup?   { �|| L   C H E C K :   d o e s   e s c a p e   d i s m i s s   t h e   p o p u p ?y }~} I   H N�p�o�p 0 pressescape pressEscape ��n� m   I J�� ?�      �n  �o  ~ ��m� I   O U�l��k�l 0 
pressenter 
pressEnter� ��j� m   P Q�� ?�      �j  �k  �m  � ��� l     �i�h�g�i  �h  �g  � ��� i   L O��� I      �f��e�f 0 iterm iTerm� ��� o      �d�d 0 filename  � ��c� o      �b�b 0 duration  �c  �e  � k     �� ��� I     �a��`�a  0 launchterminal launchTerminal� ��� m    �� ��� 
 i T e r m� ��� m    �� ���  i T e r m 2� ��_� m    �^
�^ boovtrue�_  �`  � ��� I   	 �]��\�] 0 testsequence testSequence� ��� o   
 �[�[ 0 filename  � ��Z� o    �Y�Y 0 duration  �Z  �\  � ��X� I    �W�V�U�W *0 closeterminalwindow closeTerminalWindow�V  �U  �X  � ��� l     �T�S�R�T  �S  �R  � ��� i   P S��� I      �Q��P�Q 0 terminal Terminal� ��� o      �O�O 0 filename  � ��N� o      �M�M 0 duration  �N  �P  � k     �� ��� I     �L��K�L  0 launchterminal launchTerminal� ��� m    �� ���  T e r m i n a l� ��� m    �� ���  T e r m i n a l� ��J� m    �I
�I boovtrue�J  �K  � ��� I   	 �H��G�H 0 testsequence testSequence� ��� o   
 �F�F 0 filename  � ��E� o    �D�D 0 duration  �E  �G  � ��C� I    �B�A�@�B *0 closeterminalwindow closeTerminalWindow�A  �@  �C  � ��� l     �?�>�=�?  �>  �=  � ��� i   T W��� I      �<��;�< 0 hyper Hyper� ��� o      �:�: 0 filename  � ��9� o      �8�8 0 duration  �9  �;  � k     �� ��� I     �7��6�7  0 launchterminal launchTerminal� ��� m    �� ��� 
 H y p e r� ��� m    �� ��� 
 H y p e r� ��5� m    �4
�4 boovtrue�5  �6  � ��� I   	 �3��2�3 0 testsequence testSequence� ��� o   
 �1�1 0 filename  � ��0� o    �/�/ 0 duration  �0  �2  � ��.� I    �-��,�- 0 quitterminal quitTerminal� ��+� m    �� ��� 
 H y p e r�+  �,  �.  � ��� l     �*�)�(�*  �)  �(  � ��� i   X [��� I      �'��&�' 0 	alacritty 	Alacritty� ��� o      �%�% 0 filename  � ��$� o      �#�# 0 duration  �$  �&  � k     �� ��� I     �"��!�"  0 launchterminal launchTerminal� ��� m    �� ���  A l a c r i t t y� ��� m    �� ���  A l a c r i t t y� �� � m    �
� boovfals�   �!  � ��� I   	 ���� 0 testsequence testSequence� ��� o   
 �� 0 filename  �  �  o    �� 0 duration  �  �  � � I    ��� 0 quitterminal quitTerminal � m     �  A l a c r i t t y�  �  �  �  l     ����  �  �   	 l     ����  �  �  	 

 l    �� I     ���� 0 setup setUp�  �  �  �    l   �
�	 r     J      m     �  / b i n / z s h  m     �  / b i n / b a s h � m    	 � , / o p t / h o m e b r e w / b i n / f i s h�   o      �� 
0 shells  �
  �	    l   ^ ��  X    ^!�"! k    Y## $%$ r    &&'& I    $�(�� 0 basename  ( )�) o     � �  0 	shellpath 	shellPath�  �  ' o      ���� 0 	shellname 	shellName% *+* l  ' '��������  ��  ��  + ,-, I   ' -��.���� $0 changeloginshell changeLoginShell. /��/ o   ( )���� 0 	shellpath 	shellPath��  ��  - 010 l  . .��������  ��  ��  1 232 I   . 7��4���� 0 iterm iTerm4 565 b   / 2787 m   / 099 �::  i t e r m -8 o   0 1���� 0 	shellname 	shellName6 ;��; m   2 3���� ��  ��  3 <=< I   8 A��>���� 0 hyper Hyper> ?@? b   9 <ABA m   9 :CC �DD  h y p e r -B o   : ;���� 0 	shellname 	shellName@ E��E m   < =���� ��  ��  = FGF I   B M��H���� 0 terminal TerminalH IJI b   C HKLK m   C FMM �NN  t e r m i n a l -L o   F G���� 0 	shellname 	shellNameJ O��O m   H I���� ��  ��  G P��P I   N Y��Q���� 0 	alacritty 	AlacrittyQ RSR b   O TTUT m   O RVV �WW  a l a c r i t t y -U o   R S���� 0 	shellname 	shellNameS X��X m   T U���� ��  ��  ��  � 0 	shellpath 	shellPath" o    ���� 
0 shells  �  �   YZY l     ��������  ��  ��  Z [\[ l  _ d]����] I   _ d�������� 0 teardown tearDown��  ��  ��  ��  \ ^��^ l     ��������  ��  ��  ��       ��_`abcdefghijklmnopqrstuvw��  _ �������������������������������������������������� &0 promptforpassword promptForPassword�� $0 changeloginshell changeLoginShell�� 0 record_command_with_ffmpeg  �� 0 record_command  �� 0 generate_collage  �� 0 generate_horizontal_collage  �� 0 basename  �� 0 typecommand typeCommand�� 0 
pressenter 
pressEnter�� 0 pressescape pressEscape��  0 executecommand executeCommand�� 0 	clearline 	clearLine��  0 launchterminal launchTerminal�� *0 closeterminalwindow closeTerminalWindow�� 0 quitterminal quitTerminal�� 0 newtab newTab�� 0 setup setUp�� 0 teardown tearDown�� 0 testsequence testSequence�� 0 iterm iTerm�� 0 terminal Terminal�� 0 hyper Hyper�� 0 	alacritty 	Alacritty
�� .aevtoappnull  �   � ****` �� ����xy���� &0 promptforpassword promptForPassword��  ��  x ���� 0 response  y  �� ������ ' *�� .����������
�� 
dtxt
�� 
disp
�� stic    
�� 
btns
�� 
dflt
�� 
htxt�� 

�� .sysodlogaskr        TEXT
�� 
ttxt�� 0 
passphrase  �� ��������lv���e� E�O��,E�a �� ;����z{���� $0 changeloginshell changeLoginShell�� ��|�� |  ���� 0 
executable  ��  z ���� 0 
executable  {  T�� V X�� g i�� 0 
passphrase  
�� .sysoexecTEXT���     TEXT�� ��%�%�%�%j O�%�%j b �� u����}~���� 0 record_command_with_ffmpeg  �� ����   ������ 0 sequence_name  �� 0 duration  ��  } ������ 0 sequence_name  �� 0 duration  ~  � � ���
�� .sysoexecTEXT���     TEXT�� �%�%�%�%j c �� ����������� 0 record_command  �� ����� �  ������ 0 sequence_name  �� 0 duration  ��  � ������ 0 sequence_name  �� 0 duration  �  � � �����
�� .sysoexecTEXT���     TEXT
�� .sysodelanull��� ��� nmbr�� �%�%�%�%j Olj OPd �� ����������� 0 generate_collage  �� ����� �  ���� 	0 shell  ��  � ���� 	0 shell  �  � ���
�� .sysoexecTEXT���     TEXT�� 
�%�%j e �� ����������� 0 generate_horizontal_collage  �� ����� �  �� 	0 shell  ��  � �~�~ 	0 shell  �  � ��}
�} .sysoexecTEXT���     TEXT�� 
�%�%j f �| ��{�z���y�| 0 basename  �{ �x��x �  �w
�w 
ppth�z  � �v
�v 
ppth�  � ��u
�u .sysoexecTEXT���     TEXT�y 
�%�%j g �t ��s�r���q�t 0 typecommand typeCommand�s �p��p �  �o�n�o 0 cmd  �n 0 d  �r  � �m�l�m 0 cmd  �l 0 d  �  ��k�j
�k .prcskprsnull���     ctxt
�j .sysodelanull��� ��� nmbr�q � �j UO�j h �i ��h�g���f�i 0 
pressenter 
pressEnter�h �e��e �  �d�d 0 d  �g  � �c�c 0 d  �  ��b�a�`�b $
�a .prcskcodnull���     ****
�` .sysodelanull��� ��� nmbr�f � �j UO�j i �_�^�]���\�_ 0 pressescape pressEscape�^ �[��[ �  �Z�Z 0 d  �]  � �Y�Y 0 d  � �X�W�V�X 5
�W .prcskcodnull���     ****
�V .sysodelanull��� ��� nmbr�\ � �j UO�j j �U�T�S���R�U  0 executecommand executeCommand�T �Q��Q �  �P�O�P 0 cmd  �O 0 d  �S  � �N�M�N 0 cmd  �M 0 d  � �L�K�L 0 typecommand typeCommand�K 0 
pressenter 
pressEnter�R *�jl+  O*�k+ k �J*�I�H���G�J 0 	clearline 	clearLine�I �F��F �  �E�E 0 d  �H  � �D�D 0 d  � 74�C�B�A�@
�C 
faal
�B eMdsKctl
�A .prcskprsnull���     ctxt
�@ .sysodelanull��� ��� nmbr�G � 	���l UO�j l �??�>�=���<�?  0 launchterminal launchTerminal�> �;��; �  �:�9�8
�: 
pnam�9 0 ps  �8 0 opennewwindow openNewWindow�=  � �7�6�5
�7 
pnam�6 0 ps  �5 0 opennewwindow openNewWindow� �4�3�2_Z�1�0�/�.�-�,�+�*�)�(
�4 
capp
�3 .miscactvnull��� ��� null
�2 .sysodelanull��� ��� nmbr
�1 
faal
�0 eMdsKcmd
�/ .prcskprsnull���     ctxt
�. 
prcs
�- 
cwin�, 2
�+ 
posn�*&�)�
�( 
ptsz�< W*�E/ *j UOlj O� � ���l Okj UY hO� #*�/ *�k/ ��lv*�,FO��lv*�,FUUUm �'�&�%���$�' *0 closeterminalwindow closeTerminalWindow�&  �%  �  � ���#�"�!�� 
�# 
faal
�" eMdsKcmd
�! .prcskprsnull���     ctxt
�  .sysodelanull��� ��� nmbr�$ � ���l O�j Un �������� 0 quitterminal quitTerminal� ��� �  �
� 
pnam�  � �
� 
pnam� ��
� 
capp
� .aevtquitnull��� ��� null� *�E/ *j Uo �������� 0 newtab newTab� ��� �  �� 0 d  �  � �� 0 d  � ������
� 
faal
� eMdsKcmd
� .prcskprsnull���     ctxt
� .sysodelanull��� ��� nmbr� � ���l O�j Up ���
�	���� 0 setup setUp�
  �	  �  � ������� &0 promptforpassword promptForPassword
� .sysoexecTEXT���     TEXT� *j+  O�j O�j O�j O�j q �������� 0 teardown tearDown�  �  �  � 	������ ����� 0 generate_horizontal_collage  
�  .sysoexecTEXT���     TEXT�� $0 changeloginshell changeLoginShell� **�k+ O*�k+ O*�k+ O�j O�j O*�k+ OPr ������������� 0 testsequence testSequence�� ����� �  ������ 0 filename  �� 0 duration  ��  � ������ 0 filename  �� 0 duration  � 
����=@��M��s����  0 executecommand executeCommand�� 0 record_command  �� 0 typecommand typeCommand�� 0 
pressenter 
pressEnter�� 0 pressescape pressEscape�� V*��l+ O*�%�l+ O*��l+ O*��l+ O*��l+ O*�ll+ O*kk+ 
O*�k+ 
O*�ll+ O*�k+ O*�k+ 
s ������������� 0 iterm iTerm�� ����� �  ������ 0 filename  �� 0 duration  ��  � ������ 0 filename  �� 0 duration  � ����������  0 launchterminal launchTerminal�� 0 testsequence testSequence�� *0 closeterminalwindow closeTerminalWindow�� *��em+ O*��l+ O*j+ t ������������� 0 terminal Terminal�� ����� �  ������ 0 filename  �� 0 duration  ��  � ������ 0 filename  �� 0 duration  � ����������  0 launchterminal launchTerminal�� 0 testsequence testSequence�� *0 closeterminalwindow closeTerminalWindow�� *��em+ O*��l+ O*j+ u ������������� 0 hyper Hyper�� ����� �  ������ 0 filename  �� 0 duration  ��  � ������ 0 filename  �� 0 duration  � �����������  0 launchterminal launchTerminal�� 0 testsequence testSequence�� 0 quitterminal quitTerminal�� *��em+ O*��l+ O*�k+ v ������������� 0 	alacritty 	Alacritty�� ����� �  ������ 0 filename  �� 0 duration  ��  � ������ 0 filename  �� 0 duration  � ����������  0 launchterminal launchTerminal�� 0 testsequence testSequence�� 0 quitterminal quitTerminal�� *��fm+ O*��l+ O*�k+ w �����������
�� .aevtoappnull  �   � ****� k     d�� 
�� �� �� [����  ��  ��  � ���� 0 	shellpath 	shellPath� ����������������9����C��M��V������ 0 setup setUp�� 
0 shells  
�� 
kocl
�� 
cobj
�� .corecnte****       ****�� 0 basename  �� 0 	shellname 	shellName�� $0 changeloginshell changeLoginShell�� �� 0 iterm iTerm�� 0 hyper Hyper�� 0 terminal Terminal�� 0 	alacritty 	Alacritty�� 0 teardown tearDown�� e*j+  O���mvE�O O�[��l kh  *�k+ E�O*�k+ 
O*��%�l+ O*��%�l+ O*a �%�l+ O*a �%�l+ [OY��O*j+  ascr  ��ޭ